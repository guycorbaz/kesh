# Epic 35 — choix autonomes de la story 35-8a1 (clé `e358a1`, #431, #892 item 11)

Registre tenu par l'agent de développement. Mise à jour par union, jamais par écrasement.

### C-e358a1-1 — Le module commun emporte aussi le parcours des fichiers et le découpage des arguments

- **Contexte** : D2 extrait `strip_line_comments`, `source_assainie` et `accolades_hors_chaines` vers
  `tests/common/source_assainie.rs` (pas de parseur neuf). La garde des acteurs a aussi besoin du parcours des fichiers
  de production (`racine_crates`, `fichiers_de_production`, `collecter`, `audit_label_registry.rs:151-182`) et du
  découpage des arguments d'un appel (`args_de`, `:342`) pour son cinquième motif (`delete_in_tx(…, None, …)`).
- **Retenu** : ces quatre fonctions partent dans le même module, inchangées ; les deux gardes les y prennent. Règle DRY
  du `CLAUDE.md`.
- **Écartées** : les recopier dans la garde (duplication) ; un second module (deux `#[path]` pour un même usage).
- **Réversible** : oui (tests seuls).

### C-e358a1-2 — Le module s'inclut sous le nom `assainissement`

- **Contexte** : le fichier s'appelle `source_assainie.rs` (D2) et porte une fonction `source_assainie`. Un module du même
  nom que sa fonction compile (espaces de noms distincts) mais se lit mal (`source_assainie::source_assainie`).
- **Retenu** : `#[path = "common/source_assainie.rs"] mod assainissement;` dans les deux gardes ; le nom du fichier suit
  la fiche.
- **Réversible** : oui.
