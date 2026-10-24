# Epic 32 — registre des choix autonomes de la story 32-3 (couche de services : le motif)

À verser par **union** dans `epic-32-choix-autonomes.md` au merge (`docs/agents-orchestration.md` § 1). Fiche :
`implementation-artifacts/32-3-services-motif.md`. Date : 2026-10-24. Agent de fiche : Opus 5.5, en autonomie.

---

### C-32-3-1 — Le pilote « rapports » couvre les 18 handlers, exports compris

- **Contexte** : C-32-0-4 limite la couche de services aux lectures que serviront les outils MCP ; la 32-9a ne sert pas
  d'export. Mais les neuf exports de `routes/reports.rs` partagent avec les consultations la génération **et** la trace
  (sept fonctions d'audit, 18 sites), et la 32-4b écrit « balance âgée (`reports`, déjà passé par la 32-3 : appel
  seulement) ».
- **Retenu** : les 18 handlers passent par les neuf services ; les exports gardent au handler la présentation (format,
  rendu, en-têtes, nom de fichier).
- **Écartées** : les seules consultations — un module à moitié migré, deux chemins de génération et de trace à tenir
  d'accord, et l'inventaire clos des écritures (AC11) impossible à fermer sur le module ; tous les modules de lecture
  d'un coup — c'est la 32-4.
- **Réversible** : oui.

### C-32-3-2 — `kesh-report` ne change pas

- **Contexte** : le kickoff compte `kesh-report` « seulement si une signature doit accepter une connexion plutôt qu'un
  pool ». Tous les générateurs prennent `&MySqlPool` et aucun n'écrit (fait 3 de la fiche).
- **Retenu** : `Lecture` expose son pool en `pub(crate)` aux générateurs ; aucune signature de `kesh-report` ne bouge.
  *(Amendé le 2026-10-24, validation P1 : la visibilité devient `pub(in crate::services)` — C-32-3-17 ; le reste tient.)*
- **Écartée** : passer les générateurs à `impl Executor` ou à `&Lecture` — `kesh-report` dépendrait de `kesh-api` (cycle)
  ou d'un type neuf de `kesh-db`, pour aucun gain en 32-3.
- **Réversible** : oui.

### C-32-3-3 — `Rapport<T, U>` : le contenu ne sort que par la méthode qui trace

- **Contexte** : la spike demande `Lecture` et `Trace` distinctes ; mais une trace qu'un handler peut **oublier** est
  muette (le `warn!` avale même l'échec). Les exports tracent **après** le rendu (fait 4) : tracer dans le service avant
  de rendre changerait le comportement.
- **Retenu** : le service rend `Rapport<T, Affichage|Export>` à champs privés ; `afficher(&Trace) -> T` trace puis rend ;
  `exporter(&Trace, rendre)` rend, puis trace si le rendu a réussi.
- **Écartées** : trace écrite dans le service dans tous les cas (change l'ordre des exports) ; trace laissée au handler
  par un appel séparé (oubliable) ; `#[must_use]` sur un jeton (contournable par `let _ =`).
- **Réversible** : oui ; la règle R5 (le contenu ne sort que par la méthode qui trace) est, elle, exigée.

### C-32-3-4 — Deux portées en 32-3 : `BooksRead` et `InvoicesRead`

- **Contexte** : les portées de la 0.16.0 (C-32-0-7) sont soumises à Guy (Q5 du kickoff) et persistées par la 32-5. Les
  pilotes en ont besoin de deux : les livres, et la balance âgée, rangée par la 32-9a sous `invoices:read`.
- **Retenu** : `Portee { BooksRead, InvoicesRead }` dans `services::acces` ; la 32-5 ajoute les autres et leur
  persistance. Deux variantes permettent un test qui distingue « la bonne portée » de « une portée quelconque » (M5).
- **Écartées** : les cinq portées dès 32-3 (dépend de Q5, variantes sans usage) ; une seule portée (M5 intestable).
- **Réversible** : oui (renommage si Guy change Q5).

### C-32-3-5 — Un refus est `AppError::Forbidden`, sur les deux canaux

- **Contexte** : pour l'API, le middleware RBAC rend déjà `Forbidden` (`rbac.rs:24-25`) ; pour le MCP, la 32-6 doit
  rendre un outil refusé indiscernable d'un outil inconnu (aucun oracle).
- **Retenu** : `ContexteAcces::exiger` rend `AppError::Forbidden` ; la 32-6 traduit au répartiteur.
- **Écartée** : une variante d'erreur neuve (un ajout à `errors.rs` sans consommateur en 32-3).
- **Réversible** : oui.

### C-32-3-6 — Les poignées se fabriquent par méthodes d'`AppState`, sans champ neuf

- **Contexte** : des tests construisent `AppState` par littéral (`lib.rs`, commentaire de `new_for_tests`) ; un champ
  neuf les modifierait, ce que l'AC7 interdit.
- **Retenu** : `AppState::lecture()` et `AppState::trace()`, sur `self.pool` en 32-3 ; seuls lieux de production qui
  appellent `Lecture::new` et `Trace::new` (garde de l'AC10).
- **Écartée** : un champ `pool_lecture` dès 32-3 (anticipe Q11, touche des tests). Le champ de la 32-6 touchera ces
  tests : signalé dans la fiche. *(Amendé le 2026-10-24, validation P1 R5 : « un éventuel champ » — la forme et le
  lieu de la `Lecture` en lecture seule dépendent de Q11, C-32-3-22.)*
- **Réversible** : oui.

### C-32-3-7 — La séparation se prouve par une `Lecture` de test en mode de session `READ ONLY`

- **Contexte** : en 32-3 les deux poignées partagent le pool de production ; la mutation « la trace part sur `Lecture` »
  serait invisible sans une `Lecture` qui refuse d'écrire. Q11 (forme de la lecture seule de production) est à Guy.
- **Retenu** : le test (`tests/services_motif.rs`) fabrique un pool dont chaque connexion passe `SET SESSION
  TRANSACTION READ ONLY` (sans privilège, essayé par la spike § 4.1-4.2 avec l'utilisateur `kesh`), en fait une
  `Lecture`, et la `Trace` sur le pool ordinaire. Ce n'est **pas** la preuve de production — celle-là est à la 32-6.
- **Écartées** : un utilisateur `SELECT` en test (dépend de Q11 et d'une recréation du conteneur, interdite aux agents) ;
  aucune preuve en 32-3 (la mutation de la spike resterait non tuée).
- **Réversible** : oui.

### C-32-3-8 — L'export du grand livre reste tracé en `report.generated` ; issue demandée

- **Contexte** : `export_general_ledger` (l. 362) appelle `emit_ledger_audit` : action `report.generated`, sans format —
  une piste de contrôle qui ne distingue pas l'export de la consultation (P2 au barème du `CLAUDE.md`).
- **Retenu** : conservé à l'identique (la story ne change aucun comportement), figé par l'AC9 (b) ; **issue demandée à
  l'orchestrateur** (« on trace toujours »), P2.
- **Écartée** : le corriger ici (changement de comportement, hors du critère « l'API ne change pas ») — correction
  mécanique en soi, mais pas dans cette story.
- *(Annoté le 2026-10-24, validation P1 R10/F10 : l'issue existe, **#729** (`bug`, P2), citée en `refs` ; l'assertion
  de l'AC9 (b) qui fige ce comportement porte `729` dans son nom.)*
- **Réversible** : oui.

### C-32-3-9 — La limitation L2 (`actor_type='user'` via PAT) est conservée

- **Contexte** : les traces de lecture n'emploient pas `actor_api_key_id` (commentaire l. 1422-1425).
- **Retenu** : inchangé ; `ContexteAcces` ne porte pas l'identifiant de clé en 32-3 ; l'acteur d'une clé MCP est à la
  32-5/32-6.
- **Réversible** : oui.
- *Amendé le 2026-10-24 (validation P3, F9) : même défaut que #431, qui ne nomme que les dépôts de `kesh-db` ; le
  traçage des sept fonctions d'audit des rapports (commentaire sur #431 ou issue neuve) est demandé à l'orchestrateur
  (C-32-3-31).*

### C-32-3-10 — `load_pdf_context` reste au handler, et lit par la `Lecture`

- **Contexte** : le contexte PDF (nom et langue de la société) ne sert qu'à la présentation d'un export, que le MCP ne
  sert pas.
- **Retenu** : au handler, signature `(&Lecture, company_id)` — aucune lecture de `state.pool` dans `reports.rs` (AC10).
- **Écartée** : le déplacer dans le service (de la présentation dans la couche partagée).
- **Réversible** : oui.
- *(Révisé le 2026-10-24 par C-32-3-17 : la **lecture** passe en service, la mise en forme reste au handler.)*

### C-32-3-11 — `ExportFormat` devient `services::rapports::FormatExport`, réimporté sous son ancien nom

- **Contexte** : la trace d'un export porte le format ; le service ne peut pas dépendre de `routes` ; le `mod tests` de
  `reports.rs` emploie `ExportFormat` par `use super::*` et ne doit pas changer (AC7).
- **Retenu** : le type (et ses trois méthodes) passe dans `services::rapports` ; `routes/reports.rs` l'importe par
  `use crate::services::rapports::FormatExport as ExportFormat;` — une importation du module parent est visible du
  `mod tests` par `use super::*`.
- **Écartée** : deux énumérations (une au handler, une au service) à tenir d'accord.
- **Réversible** : oui.

### C-32-3-12 — `helpers::company_for_ids`, à laquelle `get_company_for` délègue

- **Contexte** : `list_accounts` lit d'abord la société par `get_company_for(&CurrentUser, …)` (erreur `Internal`, message
  qui nomme la société et l'utilisateur) ; le service ne reçoit pas de `CurrentUser`.
- **Retenu** : extraire `company_for_ids(pool, company_id, user_id)` dans `helpers.rs`, `get_company_for` délègue — un
  seul message.
- **Écartée** : recopier la lecture et le message dans le service (DRY).
- **Réversible** : oui.

### C-32-3-13 — Aucune documentation visible ne change

- **Contexte** : refactorisation interne, comportement de l'API inchangé.
- **Retenu** : ni manuel, ni `docs/api-external.md`, ni CHANGELOG, ni README ; le motif s'écrit au doc-comment de
  `services/mod.rs`. Les passes vérifient quand même qu'aucun texte ne décrit ce qui bouge.
- **Réversible** : oui.

### C-32-3-14 — Aucun type d'Axum dans `services/`

- **Contexte** : le MCP (32-6) appellera les services hors d'un handler Axum.
- **Retenu** : pas d'`AppState`, pas d'extracteur ; seule exception, `ContexteAcces::depuis_api(&CurrentUser)`, le pont
  du canal API.
- **Écartée** : passer `&AppState` aux services (le MCP aurait accès au pool ordinaire par lui).
- **Réversible** : oui.

### C-32-3-15 — Les tests des services sont des tests d'intégration (`tests/services_motif.rs`)

- **Contexte** : les tests doivent fabriquer une `Lecture` de test et lire `audit_log` ; ils emploient l'API publique du
  module.
- **Retenu** : un fichier neuf dans `crates/kesh-api/tests/`, sur le squash `../kesh-db/test-schema` ; il porte aussi la
  garde lexicale (AC10). Aucun fichier de test existant ne change.
- **Écartée** : un `mod tests` dans `services/` (accès aux éléments privés, mais garde lexicale et fixtures mal placées).
- **Réversible** : oui.

### C-32-3-16 — Les usages s'appellent `Affichage` et `Export`

- **Contexte** : `Consultation` est déjà un rôle (`Role::Consultation`) ; un marqueur d'usage du même nom prêterait à
  confusion dans les exigences (« Consultation / BooksRead »).
- **Retenu** : `Affichage` (réponse JSON à l'écran ou à l'agent) et `Export(FormatExport)`.
- **Réversible** : oui.

---

*Choix C-32-3-17 à C-32-3-23 : remédiation de la validation P1 (2026-10-24, agent de remédiation, Opus 5.5).*

### C-32-3-17 — `Lecture::pool()` borné à la couche ; la lecture du contexte PDF passe en service

- **Contexte** : P1 (F2 MEDIUM, R12, R2) : un `pool()` en `pub(crate)` laissait un handler écrire par
  `state.lecture().pool()` sans qu'aucun volet de l'AC10 le voie ; `load_pdf_context`, privée à `routes/reports.rs`,
  lisait par ce pool et n'était pas exécutée par l'AC9 (a), qui ne voit que les services.
- **Retenu** : `pub(in crate::services) fn pool()` (et le pool de la `Trace` à même visibilité) ; le compilateur
  refuse `lecture.pool()` hors de `services` ; `rapports::identite_societe(&ContexteAcces, &Lecture)` porte le
  `SELECT` (exigence `{ Consultation, None }`), le handler garde la mise en forme ; AC10 (iii) étendu à tout `src/`
  hors `services/` et `lib.rs`, AC10 (v) contre l'élargissement (M11). Révise C-32-3-10 et C-32-3-2.
- **Écartées** : une garde lexicale « aucun `.pool()` dans `routes/` » (moins sûre que le compilateur, et
  `load_pdf_context` l'aurait contournée) ; inscrire `load_pdf_context` « hors preuve » (une lecture statique là où une
  exécution est à portée).
- **Réversible** : oui.
- *Amendé le 2026-10-24 (validation P2) : le champ de la `Trace` devient **privé**, lu par un accesseur
  `pub(in crate::services) fn pool()` ; l'AC10 (iii)-(iv) et l'ancien (v) sont remplacés par l'inventaire clos de
  l'AC10 (v) (C-32-3-24) — l'ancien (v), liste de trois formes dans un seul fichier, ne fermait pas ce qu'il disait
  fermer.* *(Annoté en P3 : l'inventaire clos est retiré à son tour, C-32-3-31 ; la visibilité reste la garde
  principale.)*

### C-32-3-18 — M1 par échafaudage nommé, sur un aiguillage commun

- **Contexte** : P1 (R3 = F3 MEDIUM) : M1 supposait un « point commun » et un `Rapport` qui garde la `Lecture` —
  ni l'un ni l'autre n'existaient dans l'esquisse.
- **Retenu** : un constructeur commun `Rapport::nouveau` et un aiguillage commun `tracer` (privés) ; M1 pose un
  champ `lecture_m1` (échafaudage, retiré à la restauration) et substitue `Trace::new(lecture_m1.pool().clone())` aux
  deux appels de `tracer` ; M1′ sur la seule branche grand livre × export. La mutation de `AppState::trace()` est
  déclarée **équivalente** en 32-3 (même pool) ; M1-témoin contrôle le test.
- **Écartées** : faire porter une `Lecture` au `Rapport` en production (met le pool de lecture à portée de la trace) ;
  une mutation dans les sept fonctions à la fois (sept éditions, et la tentation d'un écrivain à action variable,
  C-32-3-19).
- **Réversible** : oui.
- *Amendé le 2026-10-24 (validation P2, R6) : la mutation de `AppState::trace()` n'est pas « équivalente », elle est
  **non exprimable** en 32-3 (`lib.rs` ne lit pas `lecture.pool()`) ; la 32-6 la pose quand la `Lecture` aura un pool
  distinct (C-32-3-30). M1 tue désormais les dix-huit lignes de l'AC9 (b) (C-32-3-25).*

### C-32-3-19 — Les actions d'audit restent des littéraux ; les gardes de sources sont au gate ciblé

- **Contexte** : P1 (R4 MEDIUM, F9) : `audit_label_registry.rs` range parmi les « sites non résolus » tout
  `NewAuditLogEntry::user(` à action non littérale, et seule une entrée à `SITES_INDIRECTS` — un test existant, que
  l'AC7 interdit de toucher — le lèverait ; `textes_coherents.rs` (G4-ter) et `admin_pat_denied_e2e.rs` balaient aussi
  `src/`.
- **Retenu** : l'aiguillage choisit la fonction, jamais l'action ; aucune entrée à `SITES_INDIRECTS` ; les quatre
  gardes de sources (`audit_label_registry`, `textes_coherents`, `admin_pat_denied_e2e`, `audit_route_registry`) au
  gate ciblé.
- **Écartée** : un écrivain unique à action variable (moins de code, mais rouge sans remède admissible).
- **Réversible** : oui.
- *Amendé le 2026-10-24 (validation P2, F8) : les gardes de sources sont **huit**, non quatre — s'y ajoutent
  `test_schema_guard` (atteignable par le test neuf), `configuration_transmise`, `bank_account_closed_inventory`,
  `letterings_lexical` (hors d'atteinte du code prévu, lancées quand même) ; relevé et motifs d'exclusion à l'AC7.*

### C-32-3-20 — La frontière d'Axum se garde sur les lignes de code, et par un test

- **Contexte** : P1 (R1 = F1 MEDIUM) : la commande de l'AC1 attrapait la prose des doc-comments (R1 nomme `AppState`)
  et l'import de `CurrentUser` ; elle ne pouvait pas rendre « une ligne ».
- **Retenu** : lignes hors commentaires de `services/` portant `axum|AppState|CurrentUser` = exactement l'import et la
  signature de `depuis_api` dans `acces.rs` ; même règle exécutée par l'AC10 (vi) ; pas de commentaire de bloc dans
  `services/`.
- **Écartée** : édulcorer les doc-comments pour « passer » la commande.
- **Réversible** : oui.
- *Amendé le 2026-10-24 (validation P2, R8 = F7) : l'interdiction des `/* … */` est **retirée** (C-32-3-30) ; la
  définition des commentaires vaut pour tous les volets de l'AC10.*

### C-32-3-21 — `FormatExport` garde ses trois méthodes

- **Contexte** : P1 (F13 LOW) : `content_type` et `extension` sont de la présentation HTTP, que R6 laisse au handler.
- **Retenu** : les trois méthodes suivent le type dans `services::rapports` — une seule énumération, et le
  `mod tests` de `reports.rs` intouché.
- **Écartée** : deux fonctions libres ou un trait local au handler (deux lieux pour un même type, pour un gain de
  pureté seulement).
- **Réversible** : oui.

### C-32-3-22 — Rien dans la 32-3 ne fixe la forme ni le lieu de la `Lecture` en lecture seule

- **Contexte** : P1 (R5 MEDIUM) : le doc-comment prescrit à l'AC3 (« la 32-6 fait de `lecture()` une connexion en
  lecture seule »), l'AC10 (iv) et « le pool de lecture de la 32-6 » préjugeaient de Q11 — sous ses formes (b)/(c),
  l'API garde `pool` et le MCP fabrique sa `Lecture` ailleurs.
- **Retenu** : la fabrication (mode de session, utilisateur `SELECT`, ou les deux, et son lieu) est décidée par Guy
  et réalisée par la 32-6 ; la 32-3 ne suppose qu'une poignée distincte ; l'AC10 (iv) décrit l'état de la 32-3, que la
  32-6 amende en connaissance de cause.
- **Réversible** : oui.
- *Amendé le 2026-10-24 (validation P2) : l'AC10 (iv) est absorbé par l'inventaire (v-d) (C-32-3-24), qui porte la
  même réserve.* *(Annoté en P3 : (v-d) devient la garde (v-2), même réserve, C-32-3-31.)*

### C-32-3-23 — `as_of` calculé une fois, et ce que le `Rapport` expose

- **Contexte** : P1 (R7 = F7, R9 LOW) : le handler nomme le fichier après `exporter` avec des données résolues dans le
  service (période, `as_of`) ; recalculer `as_of` au handler ou au test peut franchir minuit UTC.
- **Retenu** : le `Rapport` porte `user_id` et expose en lecture seule la période résolue, les bornes du grand livre,
  `as_of`, le mode et les bornes d'un projet ; `as_of` est calculé une fois dans le service ; l'AC9 (b) lit l'attendu
  sur le `Rapport` ; la balance âgée n'est exportée qu'en CSV, trace `"format": "csv"` littérale.
- **Réversible** : oui.

---

*Choix C-32-3-24 à C-32-3-30 : remédiation de la validation P2 (2026-10-24, agent de remédiation, Opus 5.5).*

### C-32-3-24 — La frontière des poignées se garde par inventaire clos, non par formes interdites

- **Contexte** : P2 (R1 = F1 MEDIUM, **forme recyclée** du MEDIUM F2 de la P1 — signal de recyclage déclaré) :
  l'ancien AC10 (v) n'interdisait que `pub(crate)`, `pub fn pool` et `pub MySqlPool`, dans le seul `poignees.rs` ;
  `impl Deref for Lecture`, `pub(in crate)`, un accesseur d'un autre nom dans un autre fichier de `services/`,
  `Trace::new(lecture.pool().clone())` dans un service passaient tous, compilés et verts. C'est le mode d'échec que
  `CLAUDE.md` § *Inventorier les sites NON RÉSOLUS* décrit : une énumération de formes est ouverte.
- **Retenu** : champs de tuple **privés** sur les deux poignées, accesseur `pub(in crate::services) fn pool()` sur
  chacune ; fonctions d'audit **libres** recevant `&Trace` (aucun `impl Trace` hors de `poignees.rs`) ; l'AC10 (v)
  devient cinq **inventaires clos**, chacun égal à une liste écrite dans la fiche, toute ligne en plus rougissant :
  (v-a) lignes de `services/` nommant `MySql`, `Pool` ou `sqlx` — huit, sept dans `poignees.rs` et le `query_as` de
  `identite_societe` ; (v-b) en-têtes d'`impl` de tout `src/` nommant une poignée, lignes jointes — deux ; (v-c)
  lignes `fn ` de `poignees.rs` — quatre ; (v-d) `Lecture::new(`/`Trace::new(` dans tout `src/` — deux, au câblage de
  `lib.rs` (remplace les anciens (iii)-(iv), `services/` compris) ; (v-e) `Lecture(`/`Trace(` dans tout `src/` —
  deux, les définitions. Mutations M11 (gardée), M11′ (`impl Deref`), M11″ (accesseur ailleurs). Ce qui reste à la
  revue est écrit : formes non lexicales (alias de type hors de `services/`, macro).
- **Écartées** : allonger la liste des formes interdites (`Deref`, `AsRef`, `pub(in crate)`…), même faiblesse ; un
  inventaire des lignes nommant `MySqlPool` sur **tout** `src/` (108 lignes dans 24 fichiers au 2026-10-24, dont les
  modules non migrés : ingérable et sans rapport avec les poignées) ; une garde par `syn` (plus juste, mais un outil
  neuf pour une story de motif — la 32-4 peut y venir).
- **Réversible** : oui.
- *Annoté le 2026-10-24 (validation P3, R1 = F1, troisième forme recyclée) : **retiré**. Les cinq inventaires ne
  cherchaient pas `.pool()` lui-même — le seul chemin réel vers le pool —, ni son usage dans `services/`. Remplacés par
  deux gardes simples ((v-1), (v-2)) et un paragraphe *Ce que les gardes ne voient pas* ; M11′ et M11″ retirées
  (C-32-3-31). Le texte ci-dessus reste comme relevé daté.*

### C-32-3-25 — Les traces se figent sur les dix-huit couples, au service et par la route

- **Contexte** : P2 (F2 MEDIUM) : huit assertions pour huit formes laissaient dix couples (service, usage), le filtre
  de journal non nul et le mode `cumulative` sans lecture de trace ; (F3 MEDIUM) rien ne voyait un **handler**
  d'export passer par `Affichage` — la forme de #729 — ni appeler le mauvais service.
- **Retenu** : l'AC9 (b) devient une **table** de dix-huit lignes (service, usage, action, `details_json` en littéral),
  exerçant un filtre de journal, un filtre de comptes, `cumulative` et les deux formats, une assertion nommée par
  ligne ; l'AC9 (b′) rejoue la même table **par la route** ; l'AC10 (vii) exige `.exporter(` dans chaque `export_`,
  `.afficher(` dans chaque `get_`. M13 (export de la balance par `Affichage`) rougit (vii) et (b′).
- **Écartée** : (vii) seul (lexical, il ne voit pas le mauvais service) ; (b′) seul (il ne dit pas lequel des deux
  usages manque quand une trace est fausse).
- **Réversible** : oui.
- *Annoté le 2026-10-24 (validation P3, R3 = F4, F6, F8, R8) : (vii) cherche la présence hors commentaires ; (b′) est
  joué vert sur le code d'avant T3, premier commit de T2 ; (b) passe la `Lecture` en lecture seule ; (b′) rejoue la
  ligne sans bornes ; (b′) est le seul volet du mauvais **service**, le mauvais usage étant vu aussi par (vii)
  (C-32-3-31).*

### C-32-3-26 — Le refus précède la validation : un cas par service qui échoue à la validation

- **Contexte** : P2 (R3 MEDIUM, rédaction d'origine) : les cas de l'AC8 (c) (exercice ou projet **inexistants**)
  passent la validation et ne prouvent que « avant la lecture » ; un `exiger` posé entre validation et lecture
  rendrait `Validation` au MCP — un oracle que C-32-3-5 exclut.
- **Retenu** : (c-val) — `fiscalYearId = 0`, `from > to`, `projectId = 0` — et (c-lec) distincts, une assertion
  chacun ; M3′ (`exiger` après `validate_fiscal_year_id` dans `bilan`) rougit (c-val) et laisse (c-lec) vert.
- **Réversible** : oui.

### C-32-3-27 — L'exigence d'une aide partagée : le plus bas des rôles de ses routes

- **Contexte** : P2 (R5 = F5 LOW) : `identite_societe` sert neuf routes d'export (huit Consultation, une Comptable) ;
  la règle R3 (« égal au RBAC de la route ») ne pouvait pas s'y appliquer, et l'aide n'avait ni cas (c) ni mutation.
- **Retenu** : R3 complétée — une aide partagée porte le plus bas des rôles de ses routes, chaque route gardant le
  sien par son service et son middleware ; `identite_societe` reste `{ Consultation, None }` ; un cas « société
  inexistante » au MCP (`Forbidden`) et la mutation M2″.
- **Écartée** : une exigence par route appelante (une aide de présentation n'a pas à connaître ses appelants).
- **Réversible** : oui.

### C-32-3-28 — Les mesures du `span` d'export restent où elles sont

- **Contexte** : P2 (R7 = F10 LOW) : avec `exporter` (rendu puis trace), un `span.record` placé après `exporter`
  compterait l'`INSERT` d'audit dans `duration_ms` ; et la validation passée dans le service naît sous le `span`.
- **Retenu** : `byte_size` et `duration_ms` s'enregistrent dans la fermeture `rendre`, après le rendu, comme
  aujourd'hui ; la validation sous le `span` est un changement **assumé**, sans effet sur l'API (aucun champ
  enregistré sur une erreur, comme aujourd'hui pour toute erreur postérieure à l'ouverture).
- **Réversible** : oui.
- *Annoté le 2026-10-24 (validation P3, R7 = F3) : les validations qui passent sous le `span` sont **six** (l. 330,
  462, 660, 970, 1034, 1144), non deux ; trois exports n'ont pas de `span` (C-32-3-31).*

### C-32-3-29 — `ContexteAcces` à champs privés

- **Contexte** : P2 (F11 LOW) : des champs publics laissaient le futur code MCP du même crate forger
  `ContexteAcces { canal: Canal::Api { role: Admin }, … }` et passer toute exigence sans portée.
- **Retenu** : champs privés, accesseurs en lecture, construction par `depuis_api` et `mcp` seuls. Résidu écrit :
  `CurrentUser` a ses champs publics (les tests en ont besoin) et reste forgeable dans le crate ; le doc-comment le
  dit à la 32-6, dont la validation de clé est le seul constructeur légitime d'un canal MCP.
- **Réversible** : oui.

### C-32-3-30 — Arrêts mineurs de la P2

- **Commentaires de bloc** (R8 = F7) : l'interdiction des `/* … */` dans `services/` est **retirée** — la définition
  des commentaires (premier caractère non blanc `//`) lit un `/* */` comme du code, qui ne peut produire qu'un faux
  rouge, jamais masquer une ligne ; un commentaire de fin de ligne laisse la ligne compter ; la définition vaut pour
  tous les volets de l'AC10.
- **Fixtures de TVA** (R9 = F9) : `seed_accounting_company` retenu (il pose réglages, comptes de TVA due `2000` et
  d'impôt préalable `1000`, taux) ; une facture **validée** à ligne taxée `8.10`, une écriture sur `1000` ; ne pas
  recopier l'`INSERT` de réglages de `seed_open_invoice_for` (clé `company_id` déjà posée). *(F9 disait le compte
  d'impôt préalable non configuré : `test_fixtures.rs` le lie au compte `1000` — R9 avait raison.)*
- **`FormatExport`** (F12) : type `pub`, dérives `Debug, Clone, Copy, PartialEq, Eq`, trois méthodes `pub`.
- **`AppState::trace()`** (R6) : sa mutation est non exprimable en 32-3 ; à poser par la 32-6.
- **M12** (R10 = F6) : située dans la branche balance âgée de `tracer` ; « une seule variable `as_of` ».
- **`invoice_settlements`** (R11) : nommé à l'AC11 (3), fragments SQL et analyse pure.
- **Réversible** : oui.
- *Annoté le 2026-10-24 (validation P3, R3 = F4) : l'argument « un `/* */` ne peut produire qu'un faux rouge » ne vaut
  que pour les volets d'absence ou d'égalité ; (vii), volet de présence, ôte les commentaires (C-32-3-31).*

---

*Choix C-32-3-31 : remédiation de la validation P3 (2026-10-24, agent de remédiation, Opus 5.5), sur décision de
l'orchestrateur.*

### C-32-3-31 — Retrait de la preuve lexicale d'inaccessibilité du pool

- **Contexte** : P3 (R1 MEDIUM = F1 LOW) : **troisième forme recyclée** de « le pool d'une poignée est atteignable »
  — P1 F2 (visibilité `pub(crate)`), P2 R1 = F1 (formes énumérées dans un seul fichier), P3 R1 = F1 (les cinq
  inventaires cherchent des noms de type, pas `.pool()`, ni son usage dans `services/`). Chaque correction lexicale a
  ouvert une échappée nouvelle ; une garde lexicale ne peut pas prouver qu'un code Rust n'atteint pas une valeur.
- **Retenu (décision de l'orchestrateur)** : la fiche cesse de promettre cette preuve. Elle garde (1) la **visibilité
  Rust** comme garde principale — champs privés, `Lecture::pool()` et `Trace::pool()` en `pub(in crate::services)` ;
  (2) deux **gardes lexicales simples et décidables** — (v-1) aucun `.pool()` hors de `services/`, (v-2)
  `\bLecture::new\(`/`\bTrace::new\(` au seul câblage de `lib.rs` —, avec (i), (ii), (vi) et (vii) ; (3)
  l'**exécution** comme preuve de comportement — AC9 (b) sur la `Lecture` en lecture seule, dit explicitement, M1,
  M1-témoin ; (4) un paragraphe *Ce que les gardes ne voient pas* (usage de `lecture.pool()` pour écrire, de
  `trace.pool()` hors des fonctions d'audit, autre accesseur, `impl` de trait, alias, ré-export, macro,
  `unsafe`/`transmute`) : **la garantie réelle contre une écriture sur la `Lecture` est celle de la 32-6** (lecture
  seule imposée par la base, Q11), et la revue de code d'ici là — ainsi que, durablement, pour `trace.pool()`, que
  la base ne refusera pas.
- **Écartées** : un relevé de plus (`.pool()` dans `services/` par nom de récepteur, R1 proposition 1) — une convention
  de nom, contournable par un renommage, quatrième forme en attente ; `pub(in crate::services::rapports)` pour
  `Trace::pool()` (F1 proposition 1) — **non écrivable** : `pub(in …)` n'accepte qu'un module ancêtre (E0742), et
  `rapports` n'est pas un ancêtre de `poignees` ; y parvenir obligerait à définir la `Trace` dans `rapports.rs`, alors
  que la 32-5 y fera passer aussi la dernière utilisation d'une clé ; `#![forbid(unsafe_code)]` sur le crate —
  impossible (`config.rs` emploie `unsafe` dans ses tests). **Signalé à l'arbitrage, non retenu** : un
  `#![forbid(unsafe_code)]` de module en tête de `services/mod.rs`.
- **LOW de la P3 traités ici** : R3 = F4 ((vii) hors commentaires), R4 (`\b`), R6 = F2 (esquisse), R7 = F3 (six
  validations sous le `span`), R8 ((b′) et le mauvais service ; M1′ et (v-2)), R9 (M10 dans les deux fonctions), F6
  (T2 en deux commits), F7 (forme du service pour M3/M3′), F8 ((b) sur la `Lecture` en lecture seule ; ligne sans
  bornes en (b′)), F9 (L2 et #431 ; AC14 nomme `docs/MULTI-TENANT-SCOPING-PATTERNS.md` et `architecture.md`), F10
  (doc-comment de conventions rattaché à `emit_report_audit`) ; R2 au paragraphe *Ce que les gardes ne voient pas* ;
  R5 = F5 sans objet ((v-b) retiré).
- **Réversible** : oui.
- *Annoté le 2026-10-24 (validation P4) : trois points de ce choix étaient inexacts à leur date et sont rectifiés par
  C-32-3-32 — (v-1) ne cherchait que `.pool()`, l'appel qualifié `Lecture::pool(&x)` lui échappait (P4-1) ;
  « `forbid(unsafe_code)` sur le crate — impossible » ne suit pas du fait cité, les `unsafe` de `config.rs` étant tous
  sous `#[cfg(test)]`, et le `forbid` de module « signalé à l'arbitrage, non retenu » est **retenu** (P4-2) ; « #431 ne
  couvre que les dépôts » était trop étroit, et le traçage, déjà fait (P4-3). Le texte ci-dessus reste comme relevé
  daté.*

---

*Choix C-32-3-32 : remédiation de la validation P4 (2026-10-24, agent de remédiation, Opus 5.5) ; P4-2 sur décision
de l'orchestrateur.*

### C-32-3-32 — (v-1) couvre toute forme d'appel de l'accesseur ; `forbid(unsafe_code)` sur `services/`

- **Contexte** : P4 (passe ciblée, Haiku), **0/0/1/4**, tous nés de la remédiation P3. P4-1 (MEDIUM) : (v-1) ne
  cherchait que `.pool()` ; une fois `pool()` élargi, `Lecture::pool(&x)` ou `.map(Lecture::pool)` dans un handler
  compilaient et la laissaient verte, alors que la fiche disait qu'elle nommait ce rouge. P4-2 (LOW) : le rejet du
  `forbid(unsafe_code)` sur le crate reposait sur une conclusion qui ne suivait pas du fait cité.
- **Retenu** :
  - (v-1) : motif `\.\s*pool\s*\(|::\s*pool\b`, lignes de code hors commentaires, tout `src/` hors `services/` ;
    baseline vide revérifiée le 2026-10-24 ; `\bpool\b` écarté (il attraperait le champ `state.pool` et
    `self.pool.clone()`) ; témoins positifs des trois formes, témoin négatif `state.pool` ; mutation **M11-bis**
    (élargissement et `Lecture::pool(&state.lecture())` dans `get_balance_sheet` → (v-1) rouge). Restent hors
    d'atteinte, écrits à *Ce que les gardes ne voient pas* : l'appel coupé entre deux lignes entre `.`/`::` et
    `pool`, un commentaire de bloc glissé dans l'appel, un identifiant fabriqué par une macro.
  - **Décision de l'orchestrateur** : `#![cfg_attr(not(test), forbid(unsafe_code))]` en tête de `services/mod.rs`
    (AC1) ; mutation **M14** (`transmute::<Lecture, Trace>` dans `poignees.rs` → la lib ne compile pas). La forme
    `cfg_attr(not(test), …)` est celle qu'arrête l'orchestrateur ; au niveau du module, un `#![forbid(unsafe_code)]`
    nu serait aussi valide et couvrirait en plus les tests unitaires de `services/`, qui n'emploient pas `unsafe` —
    écart sans effet sur la mutation, jouée sur la lib hors `cfg(test)`. `unsafe`/`transmute` quittent *Ce que les
    gardes ne voient pas* pour `services/` et y restent **hors** de `services/`.
  - LOW : P4-3 — L2 : #431 couvre aussi des sites de route (`unvalidate`, Story 15-8b) ; le commentaire du 2026-10-24
    sur #431 nomme les sept fonctions d'audit : « tracé sur #431 ». P4-4 — (vii) : retrait des `/* … */` sur le corps
    entier, blocs de plusieurs lignes compris. P4-5 — (b′) : `crates/kesh-db/src/test_fixtures.rs:118` ; table
    paramétrée par des symboles, les dates de l'exercice seedé étant les seuls littéraux.
- **Écartées** : `#![cfg_attr(not(test), forbid(unsafe_code))]` sur **le crate** — les tests de `config.rs` n'y
  feraient pas obstacle, mais `routes/invoices.rs:1156` emploie `tokio::join!`, dont l'expansion porte un bloc
  `unsafe` (tokio 1.52.3, `src/macros/join.rs:176`, `Pin::new_unchecked`) que le lint `unsafe_code` refuserait
  vraisemblablement : un changement de tout le crate, hors du motif, que cette fiche ne peut vérifier sans compiler ;
  la 32-6 ou une story dédiée peut le reprendre. Pour la même raison, un service n'emploie pas `tokio::join!`.
- **Réversible** : oui.
- *Annoté le 2026-10-24 (validation P5, P5-1) : la prémisse des *Écartées* n'était pas établie — le lint
  `unsafe_code` ne se déclenche pas dans l'expansion d'une macro d'une autre crate (précédent `testcontainers-0.26.3`,
  `#![forbid(unsafe_code)]` et `tokio::select!` en code non-test). Le `forbid` reste **posé au module `services/`** ;
  son extension au crate n'est pas écartée pour cette raison, elle est **à essayer au développement** (T1) ; la
  contrainte « un service n'emploie pas `tokio::join!` » est retirée. P5-3 : les `unsafe` de `config.rs` sont aussi
  dans `smtp_config_tests` (l. 3293-3596), tous sous `#[cfg(test)]`. Cf. C-32-3-33.*

*Choix C-32-3-33 : remédiation de la validation P5 (2026-10-24, agent de remédiation, Opus 5.5) ; retrait prescrit par
P5-1.*

### C-32-3-33 — retrait de la prémisse `tokio::join!` ; `forbid(unsafe_code)` sur le crate à essayer au développement

- **Contexte** : P5 (passe ciblée, Sonnet), **0/0/1/2**, tous nés de la remédiation P4. P5-1 (MEDIUM) : la prémisse
  « l'expansion de `tokio::join!` porte un `unsafe` que `forbid(unsafe_code)` refuserait » n'est pas établie ; le lint
  ne se déclenche pas dans l'expansion d'une macro externe, et la crate publiée `testcontainers-0.26.3` porte
  `#![forbid(unsafe_code)]` et emploie `tokio::select!` (même forme `Pin::new_unchecked`) en code non-test (relevé par
  lecture du registre cargo local par la passe P5, non compilé). La fiche en tirait une contrainte gratuite sur les
  services — qui serait tombée sur la 32-4b (`routes/invoices.rs:1156`) — et un rejet du crate entier sans fondement.
- **Retenu** :
  - **Retrait** de la contrainte « un service n'emploie pas `tokio::join!` » (AC1, seul site vivant). La fiche
    n'affirme rien de ce que le lint fait d'une macro externe.
  - L'attribut reste **posé au module `services/`** (`#![cfg_attr(not(test), forbid(unsafe_code))]` en tête de
    `services/mod.rs`, C-32-3-32, inchangé ; M14 inchangée).
  - Son extension au crate est **à essayer au développement** — tâche T1 : poser `#![cfg_attr(not(test),
    forbid(unsafe_code))]` dans `lib.rs`, compiler ; s'il passe, le garder ; sinon le retirer et consigner au Dev Agent
    Record la cause réelle constatée (fichier, ligne, message du compilateur).
  - Ton : rien n'est affirmé comme établi qui n'ait été compilé ou lu — l'effet de l'attribut est dit raisonné, établi
    par M14 au développement (AC1, AC3, *Le motif*) ; le relevé de `config.rs` est dit « par lecture, non compilé ».
  - LOW : P5-2 — (v-1) : l'identifiant brut `r#pool` et la coupure entre `pool` et `(` rejoignent *Ce que les gardes
    ne voient pas* ; faux rouges futurs (un `AppState::pool()`, toute méthode `.pool(` d'un autre type) cités, avec la
    conduite : élargir la liste d'exceptions écrite de la garde (vide en 32-3), jamais affaiblir le motif. P5-3 —
    `config.rs` : `tests` et `smtp_config_tests`, tous deux sous `#[cfg(test)]`.
- **Écartées** : garder la contrainte « par prudence » — elle bride le rollout sans motif établi ; trancher sans
  compiler que le crate entier prend l'attribut — l'agent de fiche ne compile pas, l'essai revient au développement.
- **Réversible** : oui.
