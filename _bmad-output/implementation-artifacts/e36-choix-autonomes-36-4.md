# Epic 36 — registre des choix autonomes de la story 36-4 (clé `36-4`)

Registre propre à la story 36-4 (#1131), **versé par union**. Chaque choix : contexte, option retenue, options écartées,
réversibilité. Code lu à `9d5a142df`. Fiche : `36-4-acteur-annulation-reglement.md`. Les choix du kickoff
(`e36-choix-autonomes-e360.md`, étape 0 de 36-4 et C-e360-7) font foi ; ceux-ci les complètent.

---

### C-36-4-1 — `reconciliation_cancel::cancel_in_tx` prend un `Acteur` (fiche, D3)

- **Contexte** : C-e360-7 dit « le dé-rapprochement passe le sien » sans fixer la signature. `cancel_in_tx` reçoit
  aujourd'hui `user_id: i64, actor_api_key_id: Option<i64>` (`reconciliation_cancel.rs:580-581`) et construit un `Acteur`
  pour le seul genre `Entry` (`:744-747`).
- **Retenu** : `cancel_in_tx` prend `acteur: Acteur` à la place du couple ; la route tire `Acteur::from(&current_user)` ;
  `reconciliation.cancelled` s'écrit par `NewAuditLogEntry::par`. Patron de la 35-8a2 (`delete_in_tx`, `update`).
- **Écartée** : garder le couple et ne changer que les deux appels de `cancel_settlement_in_tx` — plus court (trois lignes
  de moins à la route et un appel de test), mais une signature `Option<i64>` de plus reste dans l'angle mort 1 de la garde.
- **Réversible** : oui ; aucun effet sur les données ni sur l'API.

### C-36-4-2 — L'enveloppe de test `reconciliation_cancel::cancel` prend un `Acteur` (fiche, D4)

- **Contexte** : C-e360-7 laisse le choix (« prend un `Acteur` ou est documentée comme telle »). L'enveloppe n'a aucun
  appelant de production ; onze appels de tests dans cinq fichiers ; elle passe `None` en position à `cancel_in_tx`, site
  nommé par l'angle mort 1 de la garde.
- **Retenu** : elle prend `acteur: Acteur` ; les tests passent `Acteur::utilisateur(u)` ; le site de l'angle mort 1
  disparaît (doc-comment de la garde : `pieges.rs` seul ; 28 lignes au lieu de 30 au `git grep`, avec la signature de
  `cancel_reconciliation_once` convertie par C-36-4-1 ; 21 lignes de paramètre au lieu de 23). *(Corrigé à la remédiation
  de la validation P1, V1-2 : ce choix disait « 29 lignes ».)*
- **Écartée** : un doc-comment « tests seuls » — laisse sur une fonction publique de `kesh-db` un `None` positionnel que
  la garde ne voit pas.
- **Réversible** : oui.

### C-36-4-3 — Les PDF des manuels se régénèrent dans la branche, puis après l'union (fiche, D8)

- **Contexte** : le fichier d'epic dit pour `user-manual.tex` « PDF régénéré après l'union, contrôlé aplati » (36-3 et 36-5
  le touchent aussi) ; la story doit pourtant prouver que le PDF dit la phrase neuve (règle « contrôler le PDF »).
- **Retenu** : la branche régénère et commite les deux PDF français, contrôlés aplatis (journal sous
  `kesh-gate-logs/e36-4/`) ; à l'intégration, l'orchestrateur les régénère après l'union des `.tex` (un conflit sur un PDF
  se résout par régénération, non par choix d'un côté).
- **Écartée** : ne toucher que les `.tex` — l'AC6 ne serait plus constatable dans la branche.
- **Réversible** : oui.

### C-36-4-4 — Le nom du fichier de la fiche diffère de la clé du registre

- **Contexte** : le prompt d'appel fixe le fichier `36-4-acteur-annulation-reglement.md` ; la clé de
  `sprint-status.yaml` est `36-4-annulation-reglement-cle-api` (kickoff).
- **Retenu** : les deux tels quels ; la fiche est nommée par le prompt, la clé n'est pas renommée (une clé renommée
  perdrait son historique au registre). La ligne de `sprint-status.yaml` nomme la fiche en commentaire.
- **Écartée** : renommer la clé ou le fichier sans décision de l'orchestrateur.
- **Réversible** : oui ; à l'orchestrateur.

### C-36-4-5 — L'aide `audit_traces.rs` porte la création de clé et compare les actions avec leur multiplicité (V2-3, V2-4)

- **Contexte** : étape 0 du développement, LOW V2-3 et V2-4 de la validation P2. Deux hôtes (`invoice_settlement_response_e2e.rs`,
  `supplier_settlement_cancel_e2e.rs`) n'incluent pas `common/mod.rs`, où vit `create_key_via_http` (`:204`) ; une
  comparaison d'actions « présentes » laisserait passer un doublon.
- **Retenu** : `create_key_via_http` est **déplacée** dans `common/audit_traces.rs` et réexportée par `common/mod.rs`
  (`pub use audit_traces::create_key_via_http;`) : ses appelants existants ne changent pas, les deux hôtes l'ont par leur
  module `common` en ligne. `assert_toutes_a_la_cle` compare la liste **triée** des actions à la liste triée attendue
  (multiplicité comprise) et nomme toutes les traces fautives.
- **Écartées** : (a) recopier la requête dans les deux hôtes (contraire au DRY, V1-5) ; (b) inclure `common/mod.rs` dans les
  deux hôtes (`mod common;`) — il tire tout le module, ses sous-modules et leurs dépendances dans deux binaires qui ne
  s'en servent pas ; (c) une assertion d'unicité séparée — la multiplicité la contient.
- **Réversible** : oui ; tests seuls.
