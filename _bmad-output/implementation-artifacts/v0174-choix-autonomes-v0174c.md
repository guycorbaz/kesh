# Release 0.17.4 — registre des choix autonomes, story v0174-correctifs (clé `v0174c`)

Registre propre à la story `v0174-correctifs` (fiche-index `v0174-correctifs.md`, fiches sœurs
`v0174-correctifs-1.md`, `-2.md`, `-3.md`), **versé par union** dans le registre de la release par
l'orchestrateur (`docs/agents-orchestration.md` § 1) ; une section d'une autre story ne se réécrit pas, un
doublon se renumérote. Chaque choix : contexte, option retenue, alternatives écartées, réversibilité.
Numéros de ligne de `7bd4d5c0` (le merge de la rétrospective de la série 0.17 sur `main`) sauf mention.

---

## Story v0174-correctifs — les bugs de l'usage réel (#819, #888, #889, #869, #872, #817, #842, #901)

### C-v0174c-1 — Découpage en trois fiches sœurs, et une fiche-index

- **Contexte** : huit issues, plus de cinq modules (`kesh-import`, `kesh-core`, `kesh-api`, `kesh-db`,
  `kesh-qrbill`, `kesh-reconciliation`, `kesh-i18n`, `frontend`) : la *Règle de splitting préventif* du
  `CLAUDE.md` s'applique avant la validation ; le prompt de la fiche demande alors des fiches sœurs
  `v0174-correctifs-<n>` sur la même branche.
- **Retenu** : une fiche-index `v0174-correctifs.md` (contraintes communes, CHANGELOG, manuels, ordre, Change
  Log commun) et trois fiches sœurs : **1** — l'import CAMT (#819 : `kesh-import`, `kesh-core`, `kesh-api`,
  trois modules) ; **2** — le serveur (#869 côté serveur, #817, #842 et #540, #901 : `kesh-api`,
  `kesh-qrbill`, `kesh-db`, `kesh-reconciliation`, quatre modules) ; **3** — les écrans (#888, #889, #872,
  #869 côté écran : `frontend`, `kesh-i18n` ; #889 est un *rollout* mécanique d'un helper, revu fichier par
  fichier). Ordre : 1 et 2 dans l'ordre qu'on veut, **3 après 2** (l'écran de #869 lit les `details` que 2
  ajoute). Clés de statut : `v0174-correctifs` (l'index, que le prompt nomme) et `v0174-correctifs-1`,
  `-2`, `-3`.
- **Écartées** : (a) une seule fiche à trois sous-stories, comme `v0172-correctifs` — le prompt demande des
  fiches sœurs dès qu'on découpe ; (b) un découpage par issue (huit fiches) — #889, #872 et #869 écran
  partagent les mêmes fichiers (`supplier-invoices/[id]/+page.svelte`), et huit boucles de revue coûtent
  sans rien isoler.
- **Réversible** : oui (fiches seulement).

### C-v0174c-2 — Aucune migration

- **Contexte** : Kesh 0.17.3 tient des livres réels ; une migration rendrait le retour à la 0.17.3
  dépendant de la sauvegarde (consigne commune) ; seule `v0174-session` peut en porter une.
- **Retenu** : aucune migration dans les trois fiches. #819 ne réécrit aucune transaction déjà importée ;
  #817 ne corrige aucune adresse stockée ; #842 agit sur le pool (code), non sur le schéma.
  `ls crates/kesh-db/migrations/*.sql | wc -l` vaut **89** avant et après (recompté à `7bd4d5c0`).
- **Écartées** : une migration de données qui relirait les transactions CAMT tronquées (#819) — le fichier
  d'origine n'est pas conservé par Kesh, rien ne permet de relire le texte entier ; une migration qui
  remplacerait les codes pays invalides (#817) — aucune valeur juste ne se devine (« SU » est-il la Suisse ?).
- **Réversible** : oui (retour à la 0.17.3 sans restauration, aucune ligne ajoutée à `_sqlx_migrations`).

### C-v0174c-3 — #819 : le texte d'un élément s'accumule, puis se distribue une fois

- **Contexte** : `parse_with_version` (`crates/kesh-import/src/camt053/mod.rs:233-248`) distribue **chaque**
  événement `Text` à `handle_text`, après `trim()` ; les `Event::GeneralRef` (références d'entité) et
  `Event::CData` tombent dans `_ => {}` (`:249`). `<Nm>Dupont &amp; Fils</Nm>` donne trois événements ; le
  dernier fragment écrase les précédents (`= Some(txt)`), ou s'ajoute comme une partie distincte (`Ustrd`,
  `ustrd_parts.push`, `:454`).
- **Retenu** : un tampon de texte `pending: String` ; `Text` (tel qu'aujourd'hui, `into_inner` puis
  `unescape`), `GeneralRef` (résolue, C-v0174c-4) et `CData` (`into_inner`, sans normalisation) **s'y
  ajoutent** ; le tampon est **vidé** — `trim()`, puis `handle_text` s'il est non vide, avec le `path`
  courant — avant le traitement de **tout autre événement** (`Start`, `End`, `Comment`, `PI`, `Decl`,
  `DocType`, `Eof`). Ainsi un élément sans entité ni CDATA produit exactement les mêmes appels qu'avant (une
  seule `Text` entre deux balises), et un commentaire au milieu d'un texte le coupe comme avant.
- **Écartées** : (a) distribuer à `End` seulement — change le comportement d'un texte suivi d'un enfant
  (contenu mixte), hors de #819 ; (b) joindre aussi à travers un commentaire — plus juste, mais change la
  sortie d'un fichier sans entité ni CDATA : la caractérisation de la v017-1 ne le couvre pas, et c'est
  précisément ce qu'il ne faut pas changer sans preuve ; (c) activer une option de `quick-xml` qui fusionne
  les références — aucune en 0.42 (`src/reader/mod.rs:317`, cité par l'issue).
- **Réversible** : oui (code).

### C-v0174c-4 — #819 : une entité inconnue refuse le fichier

- **Contexte** : XML ne connaît que cinq entités prédéfinies (`amp`, `lt`, `gt`, `quot`, `apos`) et les
  références de caractère (`&#…;`, `&#x…;`) ; toute autre exige une DTD, qu'un CAMT.053 ne porte pas.
- **Retenu** : `resolve_char_ref()` (`quick-xml-0.42.0/src/events/mod.rs:1724`) pour une référence de
  caractère, `quick_xml::escape::resolve_predefined_entity` (`src/escape.rs:851`) pour un nom ; une
  référence inconnue ou invalide rend `CamtError::MalformedXml` avec un message **rédigé par Kesh**
  (« référence d'entité inconnue : &… ; »), ajouté à `KESH_MESSAGES` (`tests/camt053_golden.rs:61-67`) et à
  l'inventaire clos `inventory_of_kesh_messages_is_closed`.
- **Écartées** : (a) ignorer l'entité inconnue — c'est le défaut même ; (b) la garder telle quelle
  (`&foo;`) — un texte faux qui paraît juste.
- **Réversible** : oui. Un fichier qui portait une entité inconnue s'importait en 0.17.3, tronqué ; il est
  désormais refusé (déclaré au CHANGELOG).

### C-v0174c-5 — #819 : la clé de doublon des relevés tient seule ; celle des lignes reçoit un alias

- **Contexte** : la détection d'un **relevé** déjà importé compare l'empreinte SHA-256 des **octets** du
  fichier (`compute_sha256_hex`, `routes/bank_imports.rs:1197`, `:1266` ; `find_by_company_and_hash`) :
  elle ne dépend pas de la lecture et tient par construction. La détection des **lignes**
  (`DuplicateKey`, `crates/kesh-core/src/bank_imports.rs:295-300`) compare date, montant, compte et la
  première référence non vide parmi `reference` → `end_to_end_id` → `transaction_id`
  (`normalize_reference_fallback`, `:373-384`), en minuscules ; le nom et le libellé n'y sont pas. Une
  référence portant une entité ou un CDATA change donc de clé : la référence de caractérisation
  `REF &amp; Co &lt;1&gt;` est stockée « 1 » en 0.17.3 (`fixtures/camt053_golden/v04_characterization.json`,
  `"reference": "1"`), elle sera lue « REF & Co <1> ». Un **autre** fichier qui reprend la même ligne
  (export qui chevauche le précédent) ne la reconnaîtrait plus : la ligne s'importerait deux fois.
- **Retenu** : le parseur garde l'ancienne lecture comme **mode hérité** (`TextMode::Legacy0173`, la boucle
  d'aujourd'hui), exposé par une fonction `#[doc(hidden)]` `kesh_import::camt053::parse_legacy_0173` ; à
  l'aperçu et à la création **CAMT** seulement, `kesh-api` relit le fichier en mode hérité, sélectionne le
  même relevé par IBAN et, si les deux lectures ont autant de transactions, passe pour chaque ligne une
  **clé d'alias** — la clé héritée, seulement quand elle **diffère** de la clé neuve — à une variante de
  `detect_duplicate_lines` qui compare la clé neuve **ou** l'alias aux clés existantes. Une ligne
  importée en 0.17.3 et reprise par un autre fichier reste un doublon signalé, ignoré par défaut.
- **Écartées** : (a) ne rien faire pour les lignes — c'est le dédoublement que le prompt demande d'éviter ;
  (b) ajouter les identifiants hérités à `ImportedTransaction` — change le type public d'un crate publiable
  pour un besoin de `kesh-api` ; (c) normaliser les clés en retirant les caractères non alphanumériques —
  change la clé de **toutes** les lignes existantes, et deux références distinctes pourraient se confondre ;
  (d) réécrire les transactions existantes — le fichier d'origine n'est pas gardé (C-v0174c-2).
- **Limite déclarée** : l'alias peut faire signaler comme doublon une ligne réellement nouvelle dont la date,
  le montant, le compte et la référence d'une ligne existante égalent sa forme tronquée (« 1 ») — signalée à
  l'aperçu, jamais écartée en silence ; le CSV n'est pas concerné (aucun XML).
- **Réversible** : oui (code).

### C-v0174c-6 — #819 : la caractérisation devient double

- **Contexte** : `parser_output_matches_references_produced_under_quick_xml_039`
  (`tests/camt053_golden.rs:191-`) exige l'égalité de la sortie avec des références produites sous
  quick-xml 0.39, régénération verrouillée (`regen_mode`, `:100-114`). Corriger #819 change la sortie de
  **la seule** fixture qui porte entités et CDATA, `v04_characterization.xml` (recherche
  `/usr/bin/grep -lE '&[a-zA-Z#]|CDATA'` sur `tests/fixtures/camt053/*.xml` et
  `frontend/tests/e2e/fixtures/camt053_*.xml` : elle, et `v04_latin1_hors_texte.xml` dont le CDATA est hors
  du texte lu et déjà en divergence admise).
- **Retenu** : (1) le **mode hérité** est comparé à **toutes** les références existantes, inchangées — c'est
  la preuve que l'alias de C-v0174c-5 reproduit exactement la lecture de la 0.17.3 ; (2) le **mode corrigé**
  est comparé aux mêmes références, **sauf** `v04_characterization`, comparée à une référence **écrite à la
  main**, nommée `v04_characterization.0.17.4.json`, dont les valeurs sont fixées par AC-1.4 de la fiche 1 ;
  l'exception est nominative, comme `UTF8_EXCEPTION`. La référence 0.39 n'est **ni régénérée ni modifiée**.
- **Écartées** : (a) régénérer la référence — interdit par le test et par sa doctrine (« ne JAMAIS
  régénérer une référence pour la faire disparaître ») ; (b) retirer la fixture — perdrait la preuve que le
  mode hérité reproduit la 0.17.3.
- **Réversible** : oui (tests).

### C-v0174c-7 — #819 : les transactions déjà importées ne se réécrivent pas ; les règles le disent

- **Contexte** : les noms, libellés et références tronqués restent tels quels en base ; les règles
  d'affectation `CounterpartyExact` (`crates/kesh-reconciliation/src/rules.rs:19`, `:49-`) écrites sur un nom
  tronqué (« Fils ») cesseront de reconnaître les imports suivants, qui portent le nom entier.
- **Retenu** : rien n'est réécrit ; le manuel utilisateur (§ *Détection des doublons* et § des règles) et le
  CHANGELOG le disent, avec le geste : vérifier les règles « nom exact » écrites sur un nom qui contenait
  « & » ou une lettre accentuée codée.
- **Écartées** : migrer les règles — aucune ne se corrige sans le nom d'origine.
- **Réversible** : oui.

### C-v0174c-8 — #869 : `details` du `PERIOD_LOCKED`, au site unique du serveur

- **Contexte** : tous les `400 PERIOD_LOCKED` de l'API naissent de `DbError::PeriodLocked { locked_through,
  attempted }`, traduit à un seul site (`crates/kesh-api/src/errors.rs:4248-4264`) par `build_response`
  (`:1367-1378`), qui ne porte que `code` et `message`. Le message nomme déjà les deux dates (clé
  `journal-entries-period-locked`, fr-CH `:438`) ; ce sont les `details` qui manquent, si bien que l'écran
  du repassage en brouillon d'une facture fournisseur — qui compose son propre texte
  (`supplier-invoice-unvalidate.ts:154-158`) — ne peut pas dire jusqu'à quand. `docs/api-external.md:493`
  écrit que `PERIOD_LOCKED` « n'émet aucun `details` ».
- **Retenu** : à ce site, un corps `{ code, message, details: { lockedThrough, attempted } }` (dates
  `AAAA-MM-JJ`), sur le patron de `later_fiscal_year_closed_response` (`:1385-`) ; les noms sont ceux que
  l'acceptation par lot porte déjà (`routes/reconciliation.rs:585-592`). Le champ est additif ;
  `docs/api-external.md` est mis à jour à chaque phrase qui parle des `details` de `PERIOD_LOCKED`.
- **Écartées** : (a) un champ propre à la route `unvalidate` — toutes les routes partagent le site ; (b)
  reformater les dates du **message** du serveur en forme suisse — défaut voisin (le texte affiche
  `2026-03-31` dans le journal et à la validation fournisseur), non demandé : **issue demandée**.
- **Réversible** : oui (champ additif).

### C-v0174c-9 — #817 : la liste ISO de `kesh-qrbill`, rendue publique, juge toute adresse écrite

- **Contexte** : `StructuredAddressInput::check_lengths` (`crates/kesh-api/src/address_input.rs:66-71`)
  ne contrôle que deux lettres ASCII ; la liste ISO 3166-1 alpha-2 vit, privée, dans
  `crates/kesh-qrbill/src/validation.rs:276` (`is_iso_3166_alpha2`, 249 codes, triée, `binary_search`).
  Les deux écrivains d'une adresse structurée : `validate_common` des contacts (`routes/contacts.rs:392`,
  création `:640`, mise à jour `:708`) et l'assistant d'installation (`routes/onboarding.rs:532`).
- **Retenu** : `is_iso_3166_alpha2` devient `pub` (réexportée par `kesh_qrbill`), et `check_lengths` la
  consulte après la forme : un code absent est refusé `400 VALIDATION_ERROR`, message qui **nomme** le code
  (« adresse : le pays « SU » n'est pas un code ISO 3166-1 alpha-2 connu (ex. CH, FR, DE) »). **À
  l'écriture seulement** : la lecture (`From<&StructuredAddress>`, `:110-`) ne valide rien et ne change pas.
  **Une adresse stockée au code invalide** se lit, se liste et s'affiche ; la modifier exige de corriger le
  pays — y compris si l'on ne touche qu'un autre champ du contact, l'écran envoyant l'adresse entière ; le
  refus le dit.
- **Écartées** : (a) recopier la liste dans `kesh-api` — deux listes divergeraient ; (b) tolérer la valeur
  stockée inchangée à la mise à jour — demande de relire la ligne avant d'écrire, et laisse vivre un code qui
  fait échouer la QR-facture ; (c) un sélecteur de pays commun aux écrans — fonctionnalité, **issue
  demandée** ; (d) un écran pour corriger l'adresse de la société — aucun n'existe
  (`routes/companies.rs` n'a ni route ni champ d'adresse en écriture, `:102`, `:240`), fonctionnalité,
  **issue demandée** : une société enregistrée en 0.17.3 avec un code invalide ne peut toujours pas le
  corriger.
- **Réversible** : oui (code).

### C-v0174c-10 — #842 : la session se remet à zéro au retour au pool

- **Contexte** : `with_account_lock` (`crates/kesh-reconciliation/src/mutex.rs:64-`) prend `GET_LOCK` sur
  la connexion d'une `Transaction` empruntée ; il rend le verrou par `RELEASE_LOCK` après la fermeture
  (`:132-136`). Si la requête est **abandonnée** pendant la fermeture (client déconnecté : Axum abandonne la
  future), ou si `RELEASE_LOCK` ne rend pas 1, la `Transaction` est abandonnée, sa connexion **revient au
  pool** (`sqlx-core-0.8.6/src/pool/connection.rs`, `return_to_pool`) avec le verrou de session —
  `GET_LOCK` survit au `ROLLBACK` —, et le commentaire « le lock advisory sera libéré à fin de session »
  (`:61-64`) est faux : la session ne finit pas, le pool la garde. Le pool de production est construit
  dans `crates/kesh-api/src/main.rs:82-85`, **sans** `create_pool` ni crochet.
- **Retenu** : un crochet `after_release` qui exécute `SELECT RELEASE_ALL_LOCKS()` (MariaDB ≥ 10.5) et
  rend `Ok(true)` ; une erreur rend `Err` (sqlx ferme alors la connexion sans la réutiliser, `close_hard`, et
  la fin de session libère le verrou). Le crochet vit dans `kesh-db` (`pool.rs`, fonction publique qui
  complète des `MySqlPoolOptions`), appliqué au pool de `main.rs` et testable sur un pool monté depuis les
  options d'une base de test. Les commentaires de `mutex.rs` (`:30-50`, `:61-64`) et ses messages de journal (« lock will leak until session end », `:150`, `:165`) sont récrits.
- **Écartées** : (a) prendre le verrou sur une connexion dédiée marquée `close_on_drop` dans
  `with_account_lock` — change la session du verrou pour ses appelants (huit appels hors tests : `routes/bank_imports.rs:1461`, `routes/reconciliation.rs:1761`,
  `:4468`, `:4966`, `:5456`, `:5789`, `:5979`, `:6282`), dont les rapprochements qui travaillent **dans** la
  transaction verrouillée, et double les connexions par requête sur un pool de 5 ; (b) un garde `Drop` —
  rien d'asynchrone ne s'exécute dans un `Drop`, et la `Transaction` empruntée ne se détache pas.
- **Coût déclaré** : une instruction de plus à chaque restitution d'une connexion au pool.
- **Réversible** : oui (code).

### C-v0174c-11 — #540 : le même crochet rétablit `FOREIGN_KEY_CHECKS`

- **Contexte** : #540 (même famille) — la restauration (`backup.rs:611-650`) désactive les clés étrangères
  sur sa connexion ; une requête abandonnée rend la connexion au pool avec `FOREIGN_KEY_CHECKS=0`. Le prompt :
  « rapprocher sans l'intégrer, sauf mécanique ».
- **Retenu** : **intégré**, parce qu'il est mécanique une fois C-v0174c-10 posé : le crochet exécute aussi
  `SET SESSION foreign_key_checks = 1`, avec un test du même modèle (pool d'une connexion). La remède diffère
  de celui que l'issue propose (`close_on_drop` après l'`acquire`) mais couvre tous les chemins, celui de la
  restauration compris. `refs #540` ; fermeture proposée à l'orchestrateur si la revue le confirme.
- **Écartées** : `close_on_drop` à l'acquisition de la restauration — ne couvre que ce site.
- **Réversible** : oui.

### C-v0174c-12 — #888 : l'écran lit toutes les pages des factures validées

- **Contexte** : `loadOpenInvoices` (`frontend/src/routes/(app)/payment-batches/+page.svelte:53-56`) lit
  une page `{ limit: 200, status: ['open'] }` ; le serveur écrête à 200 (`LIMITE_MAX`,
  `services/factures_fournisseur.rs:30`), rend `total` filtré et un ordre stable (`invoice_date DESC, id
  DESC`, `repositories/supplier_invoices.rs:252`).
- **Retenu** : lire les pages de 200 jusqu'à `total` (`offset` croissant), en s'arrêtant sur une page vide
  (garde contre une boucle si la liste rétrécit pendant la lecture) ; la liste à cocher garde sa forme. La
  création refuse déjà plus de 500 factures dans un lot (`MAX_IN_CLAUSE_IDS`, `routes/payment_batches.rs:174`)
  avec un message affiché.
- **Écartées** : (a) une pagination à l'écran — la sélection d'un lot traverserait les pages, état à tenir ;
  (b) un filtre par échéance — fonctionnalité.
- **Défaut voisin** : la liste des **lots** ne lit que les cent premiers (`listPaymentBatches({ limit: 100 })`,
  `:43`), sans signal : non mécanique (pagination de l'écran), **issue demandée**.
- **Réversible** : oui.

### C-v0174c-13 — #889 : un helper unique, `todayLocal`, déplacé dans `lib/shared`

- **Contexte** : `todayLocal(now = new Date())` existe (`frontend/src/lib/features/open-items/open-items.ts:26`,
  testé `open-items.test.ts:30`), importé hors de sa feature par `SupplierInvoiceForm.svelte:80` et
  `bank-accounts/+page.svelte:41` ; `JournalEntryForm.svelte:84-90` en porte une copie locale (`todayISO`).
- **Retenu** : `todayLocal` déménage dans `frontend/src/lib/shared/utils/local-date.ts` (avec son test) ;
  `open-items.ts` le **réexporte** (ses tests et `OpenItemsScreen` ne changent pas) ; tout site corrigé
  l'importe de `$lib/shared/utils/local-date` ; la copie de `JournalEntryForm` est remplacée (DRY). Un
  « lendemain » (clés d'API) se calcule `addDaysIso(todayLocal(), 1)`.
- **Écartées** : garder `todayLocal` dans `open-items` — un helper commun dans une feature, importé par
  quatre autres.
- **Réversible** : oui.

### C-v0174c-14 — #889 : la borne du verrou de période reste UTC, comme le serveur

- **Contexte** : `maxLockDate` (`frontend/src/routes/(app)/settings/+page.svelte:217-221`) vaut la veille
  **UTC** ; le serveur refuse une borne `>= Utc::now().date_naive()` (`crates/kesh-db/src/repositories/companies.rs:1013`,
  `:1100`). En Suisse (UTC+1 ou +2), la veille UTC n'est jamais postérieure à la veille locale : l'écran
  propose exactement ce que le serveur accepte.
- **Retenu** : site **conforme**, inchangé, inscrit à l'allowlist de la garde (C-v0174c-17) avec ce motif.
  Le serveur, lui, juge à la date UTC : entre minuit et 1 h (2 h en été), verrouiller jusqu'à la veille est
  refusé — défaut serveur, non demandé : **issue demandée**.
- **Écartées** : passer l'écran en date locale — il proposerait une date que le serveur refuse.
- **Réversible** : oui.

### C-v0174c-15 — #889 : le rappel manuel compare à la date locale

- **Contexte** : `reminderSentAtDateTime` (`frontend/src/lib/features/reminders/reminder-sent-at.ts:23-31`)
  envoie l'instant courant si la date choisie est « aujourd'hui » (en UTC), sinon midi UTC de cette date ;
  le serveur refuse un `sent_at` futur (`routes/dunning_reminders.rs:261`). Passer seulement
  `ManualReminderDialog` en date locale ferait envoyer `<aujourd'hui local>T12:00:00` entre minuit et 1 h :
  un instant futur, refusé.
- **Retenu** : les deux changent ensemble : « aujourd'hui » se juge par `todayLocal(now)` ; l'envoi reste
  `now.toISOString().slice(0, 19)` (instant UTC) pour aujourd'hui, midi UTC pour une date passée.
- **Écartées** : garder l'UTC dans le dialogue — le défaut même.
- **Réversible** : oui.

### C-v0174c-16 — #889 : les affichages d'instants, et une prémisse corrigée

- **Contexte** : le second commentaire de l'issue nomme douze affichages `formatSwissDate(x.slice(0, 10))`.
  Depuis la 0.17.2, `formatSwissDate` rend la **date à Zurich** d'un horodatage
  (`frontend/src/lib/features/reports/reports.api.ts:237-241`) : le `.slice(0, 10)` est ce qui la défait.
  Lu au code : `paidAt` (factures client et fournisseur) n'est **pas** un instant — c'est la date du
  règlement à minuit (`invoice_settlements_write.rs:323-324`, `supplier_invoices.rs:1922-1923`) ; les autres
  sont des instants UTC (`NOW(3)`, `UTC_TIMESTAMP(6)`, `CURRENT_TIMESTAMP(3)` sous `time_zone = '+00:00'`).
- **Retenu** : retirer le `.slice(0, 10)` aux **neuf** instants (`pdfFrozenAt`, `emailedAt`, `sentAt`,
  `cancelledAt`, `lastReminderAt`, `confirmedAt` ×3, `createdAt` des documents importés) ; les trois `paidAt`
  sont **conformes**, inchangés (prémisse de l'issue corrigée) ; `formatExpiryDate` des clés d'API
  (`api-keys/+page.svelte:96-103`) reste l'exception de la 0.17.2. Un site **hors inventaire**, de même
  famille et mécanique : `formatDate` de `routes/(app)/users/+page.svelte:271-279` lit un horodatage UTC
  sans décalage par `new Date(iso)` (heure locale) — remplacé par `formatSwissDate`.
- **Écartées** : renvoyer les affichages à une autre issue — ils sont dans #889 (commentaires du Project
  Lead), et la correction est un retrait.
- **Réversible** : oui.

### C-v0174c-17 — #889 : une garde Vitest tient l'inventaire clos

- **Contexte** : l'inventaire d'aujourd'hui (fiche 3, § Inventaire) se périme au premier site ajouté.
- **Retenu** : un test qui lit les sources de `frontend/src` (`readdirSync` / `readFileSync`, patron de
  `lib/shared/i18n-un-repli-par-cle.test.ts`, hors `*.test.ts` et `*.test-helper*`) et liste les lignes de `toISOString().slice(0, 10)` et de
  `formatSwissDate(<expr>.slice(0, 10))` : chacune doit figurer dans une allowlist **nominative** (fichier,
  motif écrit) — les sites conformes de l'inventaire —, et l'allowlist ne doit contenir aucune entrée
  orpheline. Sonde positive : la garde rougit sur un site réintroduit (mutation MC-889-g).
- **Écartées** : une règle ESLint — le dépôt n'en a aucune de ce genre, et un test se lit dans la fiche.
- **Réversible** : oui.

### C-v0174c-18 — #872 : le formulaire affiche les projets orphelins, et marque l'archivé

- **Contexte** : le projet est celui de l'**en-tête** de la facture (`fProjectId`, un `<select>` unique,
  `SupplierInvoiceForm.svelte:724-740`) — non celui d'une ligne, comme l'issue l'écrit (prémisse corrigée).
  La page d'édition garde le projet archivé du brouillon et écarte les autres archivés
  (`supplier-invoices/[id]/edit/+page.svelte:114`) ; le formulaire ne rend que les racines et leurs enfants
  (`:733-737`) : un sous-projet archivé dont la racine est archivée — donc écartée — n'est jamais rendu, et
  le `<select>` s'affiche sans sélection. Une racine ne s'archive qu'une fois ses enfants archivés
  (`has_active_children`, `repositories/projects.rs:141-158`, appelée `routes/projects.rs:329`), et un
  sous-projet ne se crée ni ne se désarchive sous une racine archivée (`routes/projects.rs:161-165`,
  `:338-340`) : le cas « enfant actif sous racine archivée » ne se forme pas par l'API, et les autres sélecteurs (`InvoiceForm`, `JournalEntryForm`…), qui ne lisent que les actifs,
  ne sont pas touchés.
- **Retenu** : le formulaire rend, après l'arbre, tout projet de la liste dont le parent n'y est pas
  (orphelin) ; toute option d'un projet archivé porte la marque « (archivé) » (clé neuve, quatre locales).
  La liste offerte ne change pas (un archivé n'est jamais offert comme choix neuf, C-v017-4b) ; la
  validation le refuse toujours (manuel `:1705`).
- **Écartées** : garder aussi la racine archivée dans la liste de la page d'édition — elle deviendrait un
  choix offert, que la validation refuserait.
- **Réversible** : oui.

### C-v0174c-19 — #869 : l'écran nomme la date du verrou, avec repli

- **Contexte** : C-v0174c-8 ajoute `details.lockedThrough`.
- **Retenu** : `SupplierInvoiceUnvalidateDetails` reçoit `lockedThrough?: string | null` ; le cas
  `PERIOD_LOCKED` (`supplier-invoice-unvalidate.ts:154-158`) rend une clé neuve qui nomme la date, en forme
  suisse (« … sous le verrou de période, posé jusqu'au { $date } : … »), et la clé d'aujourd'hui sans
  `details` (patron `fiscalYearName`, `:135-149`). Quatre locales.
- **Écartées** : afficher le message du serveur — il parle d'« écriture », non du geste, et porte une date
  ISO (C-v0174c-8).
- **Réversible** : oui.

### C-v0174c-20 — #901 : le CHANGELOG seul ; la cause en issue

- **Contexte** : `CHANGELOG.md:60` porte `## [0.17.1] — 2026-11-03` ; le tag `v0.17.1` est daté du
  2026-11-04 10:35 (`git for-each-ref`), la release GitHub du 2026-11-04 09:44 UTC. Les autres sections de la
  série sont justes (0.17.0 le 2026-11-03, 0.17.2 le 2026-11-04, 0.17.3 le 2026-11-05). Aucun manuel, ni le
  README, ni `website/` ne porte la date de la 0.17.1 (`git grep -n '2026-11-03'`, sur les manuels, le
  README et `website/`, ne rend rien) ; les PDF du tag `v0.17.1` portent la macro du tag, et ne se
  réécrivent pas.
- **Retenu** : la ligne du CHANGELOG passe au 2026-11-04 ; rien d'autre. La cause — `prepare-release.sh` date
  la section **à la préparation**, non au tag (`scripts/prepare-release.sh:9-10`) — n'est pas corrigée ici
  (texte seul) : **issue demandée** (ou commentaire de #901).
- **Écartées** : réécrire les notes de la release GitHub — interdit par le prompt.
- **Réversible** : oui.
