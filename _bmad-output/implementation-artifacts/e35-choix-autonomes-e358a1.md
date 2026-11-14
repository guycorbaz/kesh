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

### C-e358a1-3 — La raison finale se garde par la liste close des références de sa fonction

- **Contexte** : revue P1, B1 = E1 (MEDIUM) — la garde comptait le texte des sites, mais ni leur raison (`_` à
  `releve_declare`) ni leur appelant (non vide seulement) : une route comptable qui se mettrait à appeler une fonction de
  raison finale (`supplier_invoices::delete_draft`, `companies::unlock_books`…) ferait écrire `'user'` à une clé sans que
  rien ne rougisse, et la phrase de tête de la garde promettait l'inverse.
- **Retenu** :
  - une table `REFERENCES` dans la garde : pour chaque fonction qui porte un site de raison finale (hors `Definition`), et
    pour chaque maillon jusqu'à la route, la liste close de ses références `(fichier, fonction englobante) → nombre` ;
  - ce qui compte comme référence : le chemin qualifié écrit (appel, pointeur de fonction de `lib.rs`, import), et l'appel
    nu dans le fichier de définition ou dans un fichier qui l'importe nommément ;
  - un test compare le relevé à la liste dans les deux sens, et un autre exige que chaque site de raison finale ait sa
    fonction dans la table ;
  - sonde positive synthétique (i), et mutation M7 sur l'arbre (une route comptable de plus vers `unlock_company_books`) ;
  - 50 fonctions, 54 références, mécanisme fait du même balayage textuel que la garde, sans dépendance neuve ;
  - la phrase de tête de la garde est réécrite, et les limites du mécanisme y sont écrites (angles morts 8 et 9).
- **Écartées** :
  - le seul angle mort écrit (remède minimal) : la classe resterait ouverte pour les fonctions déjà inventoriées ;
  - une analyse d'appels par `syn` : un nouveau mécanisme, aveugle aux macros, et plus fragile que le texte ;
  - un contrôle de la raison par le routeur (`admin_routes` contre `comptable_routes` dans `lib.rs`) : il exigerait de
    découper `build_router_avec_registre` par routeur, découpage textuel fragile — le compte des références suffit à faire
    rougir une route de plus, quel que soit son routeur ;
  - étendre la chaîne `Mcp` au-delà du premier maillon : la clé MCP n'est pas une clé REST (#1057).
- **Réversible** : oui (tests seuls).
