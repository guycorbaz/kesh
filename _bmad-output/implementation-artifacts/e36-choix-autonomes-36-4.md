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
  disparaît (doc-comment de la garde : `pieges.rs` seul ; 29 lignes au lieu de 30).
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
