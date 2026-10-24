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
  tests : signalé dans la fiche.
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
- **Réversible** : oui.

### C-32-3-9 — La limitation L2 (`actor_type='user'` via PAT) est conservée

- **Contexte** : les traces de lecture n'emploient pas `actor_api_key_id` (commentaire l. 1422-1425).
- **Retenu** : inchangé ; `ContexteAcces` ne porte pas l'identifiant de clé en 32-3 ; l'acteur d'une clé MCP est à la
  32-5/32-6.
- **Réversible** : oui.

### C-32-3-10 — `load_pdf_context` reste au handler, et lit par la `Lecture`

- **Contexte** : le contexte PDF (nom et langue de la société) ne sert qu'à la présentation d'un export, que le MCP ne
  sert pas.
- **Retenu** : au handler, signature `(&Lecture, company_id)` — aucune lecture de `state.pool` dans `reports.rs` (AC10).
- **Écartée** : le déplacer dans le service (de la présentation dans la couche partagée).
- **Réversible** : oui.

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
