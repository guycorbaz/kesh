# Story 25.1b : Combler les trous d'alimentation du journal d'audit

Status: review

⚠️ **Issue du SPLIT de la 25-1** (passe 1 de validation, 2026-09-10) : **25-1a** a fermé les
chemins d'effacement et corrigé les documents publiés ; **25-1b** *(celle-ci)* comble les trous
d'alimentation ; **25-1c** livrera la route et l'écran de consultation.

⛔ **L'ordre garde son raisonnement** : une piste qu'on rend *lisible* avant d'être *complète*
donnerait à voir un journal dont les silences passeraient pour des faits. Ce que la 25-1a a fait
pour l'effacement, celle-ci le fait pour l'omission — *un journal incomplet ne ment pas moins
qu'un journal effacé, il ment plus discrètement.*

## Story

**En tant que** responsable de comptes soumis à la conservation (CO art. 957-964, OLICo art. 9),
**je veux** que les gestes qui changent les droits d'une personne ou l'identité de l'entité
laissent une trace,
**afin qu'**un réviseur puisse répondre à « qui a donné ce droit, et quand ? » — la question que
le journal existe pour trancher.

**Couvre** : [#379]. *(Voisines : [#376] et [#377] → 25-1a, mergée ; [#378] → 25-1c.)*

**Périmètre en un nombre** : **14 routes** — les 13 non tracées des six familles, plus la
**partielle** que l'inventaire comptait à part.

⚠️ **Le geste le plus sensible de l'application n'est pas comptable, il est administratif** :
`PUT /api/v1/users/{id}` peut promouvoir n'importe qui au rôle Comptable — donc au droit
d'écrire dans les livres — et n'écrit **aucune** ligne d'audit. La contre-passation et le gel
livrés par l'Epic 24 rendent les corrections *apparentes* ; ils ne disent pas **qui avait le
droit de les faire**.

## Ce que l'inventaire a établi, et qui n'était pas dans l'issue

L'issue [#379] nomme **deux** domaines ; le relevé du split en nommait **six**. L'inventaire
conduit à la spécification (2026-09-11) porte sur l'**ensemble clos des 105 routes mutantes** du
backend et rend : **73 tracées, 28 non tracées, 1 partielle, 3 sans matière à auditer.**

⚠️ **« 1 partielle » vaut pour les six familles de cette story, PAS pour le backend entier** :
[#434] en documente une seconde — `POST /onboarding/finalize`, où l'exercice comptable est tracé
mais ni les réglages de facturation ni les taux de TVA. *Un chiffre dont la portée n'est pas dite
sera relu contre le mauvais périmètre* — et il l'a été, par la passe 3.

⛔ **La méthode n'est pas négociable, et le dépôt l'a codifiée** (§ *Inventorier les sites NON
RÉSOLUS*) : on n'énumère pas les formes qui marchent, on inventorie les sites qui ne résolvent
pas. Une énumération de formes est ouverte par nature — une forme imprévue la contourne sans que
rien ne rougisse.

⛔ **La ventilation se NOMME, elle ne se résume pas à des nombres.** La « partielle » et les
« trois sans matière » restaient anonymes, donc invérifiables :

| | Route | Pourquoi |
|---|---|---|
| **partielle** *(des six familles)* | `POST /imported-supplier-invoices/{id}/complete` | la facture créée **est** auditée, la transition de la pièce ne l'est pas ⇒ **traitée ici**, AC 3 |
| sans matière | `PUT /journal-entries/{id}` | **ne mute rien** : le corps n'est même pas désérialisé, le handler rend 404 ou 409 `ENTRY_IS_POSTED` |
| sans matière | `POST /supplier-invoices/scan-qr` | parsing pur du payload SPC, **aucun accès base** |
| sans matière | `POST /bank-imports/preview` | *« parse + validate sans persistance »* — aucune mutation |

⚠️ **Le traçage se fait au REPOSITORY autant qu'à la route.** Un `grep` au niveau du fichier de
route conclut à un trou là où il n'y en a pas, et l'inverse : `companies.rs` compte deux
insertions d'audit, mais dans `lock_books`/`unlock_books`, **pas** dans `update`. L'inventaire a
donc suivi chaque route jusqu'à son repository.

### Les six familles de cette story — 13 routes non tracées, plus 1 partielle

| Famille | Routes | Ce que le silence coûte |
|---|---|---|
| `users` | 4 | **un changement de rôle vers Comptable ne laisse aucune trace** |
| `contact_persons` | 3 | le repository l'assume en tête de fichier : *« Pas d'audit log (donnée informative) »* |
| `imported_supplier_invoices` | 2 **+ 1 partielle** | une pièce entre, ressort ou aboutit sans que rien ne dise par qui |
| `companies::update` | **2**, et non 1 | l'identité de l'entité dont on tient les livres change sans trace |
| `profile` | 1 | — *(cf. AC 5 : le seul dont la valeur se discute)* |
| `setup` | 1 | **la création du tout premier administrateur** n'est pas tracée, alors que son chemin jumeau l'est |

### ⛔ Deux familles que l'inventaire a découvertes, et qui NE SONT PAS dans cette story

Elles sortent du périmètre de [#379] **et** feraient passer la story à huit modules, au-delà du
seuil de la § *Règle de splitting préventif*. Elles partent en issues propres :

- **`onboarding` — 11 routes ⇒ [#434].** Toute la séquence d'installation est invisible : création du plan
  comptable (`accounts::bulk_create_from_chart`, **non audité alors que les quatre autres
  mutations de `accounts` le sont**), création du compte bancaire, peuplement de démonstration,
  finalisation, et la remise à zéro. ⚠️ **Le code le sait déjà** : `routes/onboarding.rs:523-527`
  porte un TODO qui nomme l'absence.
- **`auth` / session — 4 routes ⇒ [#435].** `change_password` (`auth.rs:511`) n'écrit rien, alors que
  `reset_password` — **effet matériel identique** — audite à `auth.rs:948`. S'y ajoutent login,
  logout et refresh, dont **la révocation en masse pour vol de jeton détecté** (`auth.rs:382`),
  qui ne laisse aucune trace persistante.

⚠️ **Ces deux familles ne sont pas des marges.** Les écarter est un choix de découpage, pas un
verdict sur leur importance : la seconde porte le seul cas de sécurité du lot.

## Acceptance Criteria

### Volet A — les six familles

**1. `users` — quatre routes, et le changement de rôle porte son propre libellé.**
`POST /users` écrit `user.created` ; `PUT /users/{id}` écrit **`user.role_changed` si le rôle a
changé**, `user.updated` sinon ; `PUT /users/{id}/disable` écrit `user.disabled` ;
`PUT /users/{id}/reset-password` écrit `user.password_reset`. `entity_type = "user"`,
`entity_id` = l'utilisateur **cible**, l'acteur étant l'administrateur qui agit.

⛔ **Pourquoi un libellé distinct pour le rôle, et pourquoi ce n'est pas du zèle** : le dépôt a
déjà tranché ce point en séparant `books.unlocked` de `books.restored`, au motif que *« confondre
les deux rendrait le filtre d'audit inutilisable pour le réviseur qui cherche qui a déverrouillé »*
(`repositories/companies.rs:387-390`). Ici la question du réviseur est **« qui a donné le droit
d'écrire dans les livres ? »** ; noyer une promotion au rôle Comptable parmi les activations et
les changements d'adresse la rend introuvable.

⚠️ **`details_json` doit porter l'avant ET l'après du rôle** — `{"before": {"role": …, "active": …},
"after": {…}}`. L'ancien état est déjà en main : le handler lit la cible en tête
(`routes/users.rs:229-231` et `:289-291`).

⛔ **Jamais de mot de passe ni de hachage**, ni en clair ni haché — ni `req.password`, ni
`new_hash`, ni `user.password_hash`. Pour `user.password_reset`, **`details_json = None`** : le
précédent du dépôt est `routes/auth.rs:948-957`, et l'acteur et la cible suffisent à tout dire.

⚠️ **Deux libellés coexisteront pour un effet matériel identique** : `user.password_reset`
(réinitialisation **par un administrateur**) et `auth.password_reset_completed` déjà écrit par le
chemin self-service (`routes/auth.rs:951`). C'est **voulu** — l'un est subi, l'autre demandé, et
le réviseur qui cherche « qui a changé le mot de passe de qui » a besoin de les distinguer —
**mais il doit chercher les deux**, et c'est écrit ici pour qu'il le sache.

**2. `contact_persons` — trois routes.** `contact_person.created`, `contact_person.updated`,
`contact_person.archived`, `entity_type = "contact_person"`.

⚠️ **Le commentaire d'en-tête du repository devient faux et doit partir** :
`repositories/contact_persons.rs:5` affirme *« Pas d'audit log (donnée informative) »*. Le laisser
serait la faute de la 25-1a répétée — *un module qui affirme le contraire de ce qu'il fait est un
mensonge qui survit à la story qui l'a créé.*

⚠️ **`DELETE` ne connaît rien de ce qu'il archive** — le handler n'a que l'identifiant
(`routes/contact_persons.rs:209-216`). Une trace lisible exige un pré-chargement
(`contact_persons::find_by_id_in_company`), sur le modèle de `contact.archived` qui journalise un
instantané complet (`repositories/contacts.rs:656-667`).

**3. `imported_supplier_invoices` — une trace PAR PIÈCE, pas par lot, et QUATRE transitions.**
`imported_supplier_invoice.created` à l'entrée d'une pièce, `.reactivated` à sa réintroduction,
`.completed` à sa transformation en facture, `.discarded` à son rejet ; `entity_id` = la pièce.

⛔ **`.completed` est la « route partielle » de l'inventaire, et son absence était une
ASYMÉTRIE** : `POST /imported-supplier-invoices/{id}/complete` audite la facture fournisseur
créée (`supplier_invoice.created`, via `supplier_invoices::create_in_tx`) mais **pas** la
transition de la pièce (`mark_completed`). Le cycle de vie d'une pièce disait donc `created` →
`discarded` quand elle est rejetée, et `created` → **rien** quand elle aboutit. ⚠️ *Le coût est
d'une ligne* : la transaction est déjà ouverte au handler et `mark_completed` la prend déjà.

⛔ **Pourquoi par pièce alors qu'une trace de lot coûterait dix fois moins** : une trace de lot
n'est pas atteignable par `find_by_entity("imported_supplier_invoice", id)` — donc **invisible à
l'écran que livrera la 25-1c**, qui lit la piste par entité. Une trace que la consultation ne
montre pas ne répond à personne. Le prix est de faire descendre `(user_id, api_key_id)` le long
de `run_inbox_import` → `process_inbox` → `process_one_file` ; c'est mécanique.

⚠️ **Conséquence à écrire, pas à découvrir** : un import qui n'accepte **aucune** pièce n'écrit
alors **aucune** trace. C'est voulu — rien n'est entré dans le système.

⛔ **TROIS chemins d'entrée, pas deux — le troisième est une RÉACTIVATION.** Quand le hash d'un
fichier correspond à une pièce `discarded`, `inbox_import.rs:418-431` la repasse en `to_complete`
par `reactivate_to_complete` et rend **`FileOutcome::Accepted`** : la pièce figure au rapport des
acceptées, indiscernable d'une création. ⇒ **`imported_supplier_invoice.reactivated`**, et
`reactivate_to_complete_in_tx` à extraire comme `create_in_tx` — même défaut, fonction voisine
(`repositories/imported_supplier_invoices.rs:211`, `.execute(pool)` à `:223`).

⛔ **Sans lui, la piste dirait le CONTRAIRE de l'état** : une pièce écartée puis réintroduite
garderait `imported_supplier_invoice.discarded` pour **dernière** trace alors que son statut est
`to_complete`. *Un journal qui affirme le contraire de la base est pire qu'un journal muet.*

⚠️ **Et la réciproque de la phrase ci-dessus devient fausse si on l'oublie** : un import peut
**accepter** une pièce et n'écrire aucune trace.

⛔ **L'acteur ne suffit pas non plus : il manque une transaction.** `imported_supplier_invoices::create`
(`repositories/imported_supplier_invoices.rs:28`) exécute son `INSERT` **directement sur le pool**
(`:61`), et **aucun variant `create_in_tx` n'existe** — à la différence de `mark_completed`
(`:144`) et `mark_discarded` (`:177`), qui prennent tous deux une transaction. Faire seulement
descendre l'acteur laisserait la trace s'écrire dans une transaction **distincte**, après
l'autocommit de l'insertion : *exactement la fenêtre non atomique que l'AC 9 ferme sur les douze
autres routes.* ⇒ **extraire `create_in_tx` et faire porter la transaction par
`process_one_file`**, qui reçoit déjà le pool et n'a qu'un seul appelant.

⛔ **`creditor_iban` est un IBAN complet** : appliquer la convention du dépôt — `"iban_present":
true`, jamais la valeur (`routes/bank_accounts.rs:460-461`).

**4. `companies` — deux routes, et c'est l'effacement silencieux qui justifie la trace.**
`company.updated`, `entity_type = "company"`, `entity_id` = la société ; `details_json` ne porte
**que les champs que la route touche** (`email` d'un côté, `phone` et `website` de l'autre), en
`{before, after}`.

⚠️ **Ne pas journaliser l'objet entier** : `companies::update` est un *full-replace* et les
handlers reconstruisent la totalité des champs depuis l'état courant — un `{before, after}`
complet dirait que quinze champs ont changé quand un seul a bougé.

⛔ **C'est ici que la trace a le plus de valeur** : sur `/contact-details`, **une clé absente vaut
`null` et efface** (`routes/companies.rs:155-163`). Un effacement par omission ne laisse
aujourd'hui aucune trace — journaliser l'avant et l'après le rend opposable.

**5. `profile` — la trace porte sur l'INSTALLATION, pas sur l'utilisateur.**
`PUT /profile/mode` écrit `installation.ui_mode_changed`, `entity_type = "installation"`,
`entity_id = AUDIT_ENTITY_ID_NONE`, `details_json = {"before": …, "after": …}`.

⛔ **C'est le seul critère dont la valeur se discutait, et l'inspection la tranche** : `ui_mode`
n'est **pas** une préférence d'affichage par utilisateur. Il vit dans `onboarding_state`, table
**mono-ligne et globale, sans `company_id`** (`entities/onboarding.rs:60-72`) — un utilisateur
bascule donc l'installation entière en mode Expert, ce qui ouvre à tous l'écriture directe au
journal. ⚠️ **Écrire `entity_type = "user"` ici serait mentir sur la portée.**

⚠️ **Cette route écrira une trace même quand rien ne change, et c'est assumé.**
`onboarding::update_step` incrémente `version` **inconditionnellement** — aucun court-circuit
no-op —, donc un `PUT` répété avec le même mode écrira `installation.ui_mode_changed` avec
`before == after`. L'AC 8 ne s'y applique pas et sa garde `version` n'y servirait à rien.
*Une décision subie devient un défaut ; écrite, elle reste une décision.*

⚠️ **Le handler n'a pas d'acteur** : `set_mode` n'extrait aucun `CurrentUser`
(`routes/profile.rs:23-27`) — c'est la seule route authentifiée du lot dans ce cas. L'extracteur
s'ajoute au handler, le routage ne bouge pas.

**6. `setup` — la création du tout premier administrateur, acteur et cible confondus.**
`POST /setup/admin` écrit `user.created` avec `entity_type = "user"`, `entity_id = user.id`, et
`details_json` portant **`"first_admin": true`** — ce qui distingue ce geste fondateur d'une
création ordinaire sans inventer un second libellé pour la même chose matérielle.

⛔ **L'acteur est l'utilisateur créé lui-même** — la route est publique, montée avant tout
`require_auth` (`lib.rs:965`), donc il n'existe ni `CurrentUser`, ni jeton, ni identité antérieure.
C'est le patron déjà tenu par `auth/bootstrap.rs:252-265` et `routes/auth.rs:948-957`.

⛔ **Le premier administrateur a DEUX chemins de création, et AUCUN n'est tracé.**
`auth/bootstrap.rs:123` le crée depuis `KESH_ADMIN_PASSWORD` par `users::create`, et le seul
`insert_in_tx` de ce fichier est à `:252` — le *break-glass*, pas la création. ⇒ **le chemin
d'amorçage est tracé lui aussi**, avec le même libellé et le même `"first_admin": true`.

⚠️ **Sans cela, la réponse à « qui a créé le premier administrateur ? » dépendrait SILENCIEUSEMENT
du chemin d'installation.** Et ce chemin n'est pas une route : il échappe aux 105, à [#434] comme à
[#435] — *il ne serait donc écrit nulle part.*

⚠️ **Contrainte d'ORDRE, non négociable** : l'audit se pose **après** `users::create_in_tx` et
**dans la même transaction**. `insert_in_tx` remplit `actor_label` par un sous-`SELECT` sur
`users` (`repositories/audit_log.rs:79-80`) ; posé avant l'insertion, il écrirait `'(inconnu)'`.

⚠️ **Aucune trace sur les chemins refusés, mais pour DEUX raisons distinctes** : les deux sorties
`410` (`setup.rs:183-188` et `:212-221`) et la sortie `500` (`:224`) sont **après** l'ouverture de
la transaction et font `rollback` — c'est elle qui garantit leur silence. Les quatre `400`
(`:91`, `:98`, `:108`, `:121`) sont **avant** le `begin` : ils sont muets parce que le code
n'atteint jamais l'audit. *Attribuer le second silence à la transaction serait s'appuyer sur un
mécanisme qui n'opère pas là.*

### Volet B — les invariants qui valent pour les quatorze

**7. Le constructeur dit la vérité sur l'acteur.** `NewAuditLogEntry::user` **uniquement** là où un
jeton d'API ne peut structurellement pas passer — le bloc `admin_routes`, seul porteur de
`require_not_pat` (`lib.rs:330`) — ou en l'absence totale de `CurrentUser` (`setup`). Partout
ailleurs : `from_current_user` quand le handler a l'acteur, `for_actor` quand l'appel est plus bas.

| Routes | Constructeur |
|---|---|
| les quatre `users`, les deux `companies` | `::user` — le bloc est fermé aux jetons |
| `contact_persons` ×3, `discard`, **`complete`** | `from_current_user` |
| `inbox-import` | `for_actor`, `(user_id, api_key_id)` **threadés** |
| `profile/mode` | `from_current_user`, après ajout de l'extracteur |
| `setup/admin` | `::user(user.id, …)` — pré-authentification |

⛔ **Employer `::user` hors de ces cas n'est pas une approximation, c'est écrire un fait faux** :
`actor_type = 'user'` sur une action faite par une intégration. C'est la dette [#431], et une
story qui la reproduit l'aggrave.

**8. Une opération sans changement n'écrit pas de trace.** **Quatre** des quatorze routes passent par
un repository qui **court-circuite le no-op** — les deux `users` (`PUT /users/{id}` et `/disable`,
qui partagent `update_role_and_active`) et les deux `companies` (`/email` et `/contact-details`,
qui partagent `update`) et retourne l'état antérieur sans incrémenter
`version` — `users::update_role_and_active` (`repositories/users.rs:384-387`) et
`companies::update` (`repositories/companies.rs:176-186`, le `if` à `:181`). L'audit n'est écrit **que si `version` a
bougé**, sur le patron de `routes/bank_accounts.rs:741`.

⚠️ *Sans cette garde, un `PUT` identique écrirait une trace qui affirme un changement qui n'a pas
eu lieu — un journal qui invente des faits est pire qu'un journal muet.*

**9. La trace et la mutation sont atomiques.** L'écriture d'audit partage la transaction de
l'opération auditée : une mutation dont la trace ne peut pas s'écrire **n'est pas commitée**. Là
où le repository ouvre et commite seul, la story extrait un variant `_in_tx` et le handler mène la
transaction — elle n'ajoute **pas** une transaction séparée pour l'audit d'une mutation.

⚠️ **Cela vaut aussi pour `setup/admin`**, où l'échec de l'audit ferait échouer la création du
premier administrateur. C'est le comportement voulu, et il est assumé ici plutôt que découvert en
production.

**10. Aucun secret dans `details_json`.** Ni mot de passe, ni hachage, ni jeton, ni IBAN complet.
Les booléens de présence (`"iban_present": true`) sont la forme du dépôt. Les clés sont en
**snake_case** — la surface HTTP est en camelCase, la piste ne l'est pas
(`routes/reports.rs:1350-1367`).

### Volet C — que l'inventaire reste vrai après la story

**11. Les routes mutantes sont inscrites à un registre, et chacune y est soit tracée, soit exemptée
avec sa justification.** Un test de garde extrait de `lib.rs` **chaque route mutante par son
identité** — verbe et handler — et en fait un **diff ensembliste** avec le registre : il échoue en
nommant toute route absente du registre, et toute entrée du registre disparue de `lib.rs`.

⚠️ **L'ensemble clos est celui de `lib.rs`, PAS celui du backend.** Trois routes mutantes vivent
ailleurs — `/seed`, `/reset` et `/password-reset-token` (`routes/test_endpoints.rs:49,50,53`),
montées par le `nest()` de `lib.rs:995` et conditionnées au mode test. ⇒ **l'extracteur lit `lib.rs` ET `routes/test_endpoints.rs`**, et le registre les inscrit
nommément.

⚠️ **Elles sortent du SECOND volet du diff seulement.** « Route absente du registre » s'applique
aux deux fichiers — une quatrième route ajoutée à `test_endpoints.rs` doit rougir. « Entrée du
registre disparue de `lib.rs` » ne s'applique qu'à `lib.rs`, faute de quoi ces trois entrées
échoueraient en permanence. *Un garde qui rougit toujours est un garde qu'on désactive — mais un
garde qu'on exempte entièrement ne garde plus rien.* *Un détecteur dont on croit à tort qu'il couvre tout est pire qu'un détecteur
absent.*

⚠️ **L'extracteur doit échouer bruyamment si deux routes partagent verbe et handler.** L'identité
est aujourd'hui discriminante — 105 couples, zéro doublon, vérifié — mais **rien ne l'impose** : un
alias futur ferait disparaître une route du diff en silence. Il doit par ailleurs s'ancrer sur
`routes::`, `lib.rs:1015-1016` portant des constructeurs dans un bloc **commenté**.

⛔ **Une égalité de cardinalité ne suffit pas, et le croire serait le défaut que ce critère
prétend fermer** : une route retirée pendant qu'une autre est ajoutée laisse le compte inchangé et
la dérive invisible. Le précédent `admin_pat_denied_e2e` peut compter, lui, parce qu'il opère sur
un **bloc clos entre marqueurs** ; les 105 routes sont réparties dans tout le fichier. *Un
détecteur mal formé coûte le même diagnostic qu'un défaut réel.*

⛔ **Pourquoi un registre et non une liste dans la documentation** : cette story ferme **13** des
**28** sites non tracés. Les **15 restants** — `onboarding` et `auth` — deviennent des angles morts
**assumés**, et un angle mort qui n'est écrit nulle part redevient un oubli au premier ajout de
route. Le dépôt connaît ce mécanisme et l'a outillé deux fois : `EXEMPT_MIGRATIONS`
(`post_restore.rs`) et le garde-fou de schéma de test. ⚠️ **Un inventaire exact qui ne vit que dans
un story file se périme le jour où la story est close.**

⚠️ **Ce que ce test NE fait PAS, et il faut l'écrire** : il ne peut pas vérifier *qu'une route est
réellement tracée* — le traçage se fait au repository, parfois à trois appels de distance, et
aucune analyse statique raisonnable ne le suit. Il vérifie que **toute route mutante a été
examinée**, ce qui est une propriété plus faible et la seule qui soit décidable. Le contrôle du
contenu reste le fait des tests par route des AC 1 à 6.

**12. Les manuels administrateur ET utilisateur disent ce que l'inventaire établit.** **Huit**
affirmations sont corrigées, et les trois PDF régénérés :

**HUIT sites, dans DEUX manuels** — trois à l'inventaire, deux à la passe 2, un à chacune des
passes 3, 4 et 5.

⛔ **Les sept premiers ont été trouvés en posant chacun une question neuve** — couverture, champs,
attribution, exportabilité. **Le huitième, non** : il répond à une question déjà posée, dans un
**fichier qu'elle n'avait pas visité** — le glossaire du manuel utilisateur. ⚠️ *Un inventaire
documentaire ne se clôt ni par l'épuisement des fichiers, ni par celui des questions, mais par leur
PRODUIT.*

| Site | Ce qu'il dit | Ce qui est vrai |
|---|---|---|
| `admin:1782` | *« **Toute** action métier mutante est enregistrée »* | **28 routes sur 105 ne l'étaient pas** — et **15 ne le seront toujours pas** après cette story ([#434], [#435]) |
| `admin:1786` | champs `user_id, company_id, timestamp, action, entity_type, entity_id, metadata_json` | **trois sont faux** : `company_id` **n'existe pas**, `timestamp` est `created_at`, `metadata_json` est `details_json` — et quatre manquent (`id`, `actor_type`, `actor_api_key_id`, `actor_label`) |
| `admin:1956` | *« la couverture n'est pas complète : **la gestion des utilisateurs** et quelques autres »* | la réserve reste **vraie**, mais son **exemple devient faux** : il doit nommer `onboarding` et `auth` |
| **`admin:2184`** *(glossaire)* | *« Trace de **toutes** les actions métier »* | même promesse que `:1782`, **400 lignes plus loin** — la corriger seule **réinstallerait la contradiction** |
| **`user-manual:1591-1594`** | *« de **toutes** les actions comptables significatives — … **changements de paramètres** »* | **un autre manuel**, et l'exemple le plus faux du lot : la création du plan comptable par l'onboarding n'est pas tracée ([#434]) |
| **`admin:1762`** | *« les mutations effectuées via une clé sont tracées avec `actor_type = 'api_key'` »* | ⛔ **faux pour les 38 sites de [#431]**, qui écrivent `'user'` — et cette story **ne les corrige pas** |
| **`user-manual:1733`** *(glossaire)* | *« Journal de **toutes** les modifications comptables »* + un renvoi vers une section *« Journal d'audit »* **qui n'existe pas** (elle s'appelle « Traçabilité ») et dont les deux réserves portent sur l'**inaltérabilité**, pas la couverture | même promesse que `:1591-1594`, dans le **glossaire de l'autre manuel** — la corriger seule réinstallerait la contradiction |
| **`admin:1935`** | *« al. 3 : … → garanti par `audit_log` + SHA-256 + format ouverts du **ZIP d'export** »* | ⛔ **`audit_log` n'est PAS dans l'export** (19 tables sur 38, [#386]) — une affirmation de **conformité OLICo** gagée sur un contenu inexistant |

⛔ **Le sixième site ne parle pas de COUVERTURE mais d'ATTRIBUTION**, et c'est pourquoi cinq
relectures successives l'ont manqué : on cherchait « ce qui est tracé », il dit « comment c'est
attribué ». ⚠️ *Un inventaire construit sur une seule question ne trouve que les réponses à
cette question.*

⛔ **Le `:1782` n'est pas rendu faux par cette story — il l'était déjà, et il le restera.** C'est
précisément pourquoi il se corrige **ici** : cette story est le seul moment où l'on sait
exactement ce qui est tracé et ce qui ne l'est pas. La 25-1a a posé la règle — *différer une
promesse fausse revient à la maintenir.*

⚠️ **Et le manuel se contredit lui-même à 174 lignes d'écart** : `:1782` promet la couverture
totale, `:1956` avoue qu'elle est partielle. C'est le motif exact que la 25-1a a payé sur la
section de conformité OLICo.

⚠️ **`company_id` mérite mieux qu'une suppression** : la colonne **est décidée** (arbitrage du
2026-09-11) et arrivera avec la 25-1c. Écrire qu'elle n'existe pas encore, plutôt que l'effacer
comme une erreur de plume.

## Tasks / Subtasks

- [x] **T1 — Écrire la décision de conception AVANT de coder** (AC 9)
  - [x] Pour chacune des 14 routes, arrêter où l'audit se pose : handler menant la transaction
        (variant `_in_tx`) ou repository. Le tableau des Dev Notes donne la décision proposée ;
        la confirmer ou la contester **par écrit**, avec le motif.
  - [x] ⚠️ Avant de toucher la signature de `companies::update` ou de `onboarding::update_step` :
        `grep -rn "companies::update(\|onboarding::update_step(" crates/` — **les deux ont
        d'autres appelants**. Le geste sûr est d'extraire un `_in_tx` et de laisser la fonction
        actuelle en mince enveloppe.
- [x] **T2 — `users`, quatre routes** (AC 1, 7, 8, 9, 10)
  - [x] `POST /users` : `create_in_tx` existe déjà (`repositories/users.rs:50`) — le handler mène
        la transaction.
  - [x] `PUT /users/{id}` et `/disable` : extraire `update_role_and_active_in_tx`. Les deux routes
        partagent la fonction et doivent écrire **des libellés différents** — c'est le handler qui
        sait lequel, pas le repository.
  - [x] `/reset-password` : `update_password_in_tx` existe (`repositories/users.rs:320`), et son
        doc-comment dit qu'il a été créé pour cela.
  - [x] ⚠️ **La révocation des sessions reste HORS de la transaction** : `routes/users.rs:267`,
        `:317` et `:344` appellent `refresh_tokens::revoke_all_for_user(&state.pool, …)` après le
        commit. Elle prend le pool — donc une seconde connexion sur les cinq. Ne pas chercher à
        l'y faire entrer, mais **l'écrire** plutôt que de la laisser découvrir.
  - [x] Tests greffés sur `crates/kesh-api/tests/users_e2e.rs` — `update_user_change_role` y est
        **déjà écrit**, il suffit de lui ajouter l'assertion d'audit.
  - [x] ⛔ **Tracer aussi le chemin d'amorçage** `auth/bootstrap.rs:123` (AC 6) — il crée le même
        premier administrateur et n'écrit rien.
  - [x] ⚠️ **Sa branche `UniqueConstraintViolation` change de forme** : l'appel vit dans un `match`
        dont une branche nettoie la société stub. Ouvrir la transaction, n'auditer que sur `Ok`,
        laisser le `Drop` rollbacker sinon, et **conserver le nettoyage existant**.
- [x] **T3 — `contact_persons`, trois routes** (AC 2, 7, 9, 10)
  - [x] Variants `_in_tx` : aucune de ces trois fonctions n'ouvre de transaction aujourd'hui.
  - [x] Pré-charger l'instantané avant l'archivage, sans quoi la trace ne nomme personne.
  - [x] ⚠️ Corriger l'en-tête `repositories/contact_persons.rs:5`.
  - [x] ⛔ **Créer `crates/kesh-api/tests/contact_persons_e2e.rs`** : ces trois routes n'ont
        **aucun test**, ni d'intégration ni E2E. *C'est un trou de couverture que la story découvre
        et qu'elle ne peut pas laisser* — on n'ajoute pas une trace à du code que rien n'exerce.
- [x] **T4 — `imported_supplier_invoices`, trois routes** (AC 3, 7, 9, 10)
  - [x] ⛔ **Extraire `imported_supplier_invoices::create_in_tx`** — le `create` actuel écrit
        **sur le pool** (`:61`), donc l'audit ne pourrait pas partager sa transaction. Sans cette
        extraction, l'AC 9 est **intenable sur cette seule route**.
  - [x] ⛔ **Extraire aussi `reactivate_to_complete_in_tx`** — le chemin de réactivation
        (`inbox_import.rs:418-431`) porte le **même** défaut, à soixante lignes de là.
  - [x] ⚠️ **La transaction se pose autour de la SEULE étape d'insertion** (`inbox_import.rs:488-516`),
        **pas** en tête de `process_one_file` : le pool est à **5 connexions**
        (`main.rs:73`), `run_inbox_import` en tient déjà une pour son `GET_LOCK`, et la tenir
        pendant le `sleep` de stabilité, le rendu PDF et l'archivage disque donnerait une connexion
        *idle-in-transaction* par fichier — jusqu'à 200 par run.
  - [x] ⚠️ `tx.rollback()` **avant** `dispose_failed` sur les branches `UniqueConstraintViolation`
        (`:495`) et `DataLengthOrRange` (`:500`).
  - [x] ✅ **Vérifié en passe 2, à ne pas re-débattre** : le `GET_LOCK` n'interfère pas (verrou
        nommé, connexion distincte) ; une transaction **par fichier** est le bon grain — un échec ne
        rollbacke que sa pièce, les précédentes restent commitées ; `create` n'a qu'un appelant de
        production.
  - [x] Faire descendre `(user_id, api_key_id)` sur `run_inbox_import` → `process_inbox` →
        `process_one_file`, puis `for_actor` à l'endroit de l'insertion.
  - [x] `discard` **et `complete`** : la transaction est **déjà ouverte** dans les deux handlers —
        l'appel se glisse après `mark_discarded` et après `mark_completed`.
  - [x] Tests greffés sur `tests/inbox_import_e2e.rs` (`discard_marks_discarded` existe).
- [x] **T5 — `companies`, deux routes** (AC 4, 7, 8, 9, 10)
  - [x] Extraire `companies::update_in_tx` ; garder le court-circuit no-op **dans** le variant, et
        la garde `version` **dans** le handler.
  - [x] Tests sur `tests/companies_e2e.rs` : `an_omitted_field_clears_it_just_like_null` prouve que
        l'effacement silencieux laisse désormais une trace, et
        `overlong_contact_details_are_rejected_by_the_api` qu'un refus n'en laisse aucune.
- [x] **T6 — `profile`** (AC 5, 7, 9, 10)
  - [x] Ajouter l'extracteur `Extension(current_user)` et capturer le résultat de
        `update_step_in_tx`, que le handler jette aujourd'hui.
  - [x] Extraire `onboarding::update_step_in_tx` ; `update_step` reste une **enveloppe mince et
        NON auditée**, et le handler mène la transaction.
  - [x] ⛔ **L'audit se pose dans le HANDLER, jamais dans `update_step`.** Cette fonction a
        **d'autres appelants** — le seed et huit routes d'onboarding, toutes hors périmètre et
        suivies par [#434] : y placer la trace ferait écrire `installation.ui_mode_changed` à
        **chaque étape de l'onboarding**. *Une trace au mauvais étage ne manque pas : elle ment.*
  - [x] Tests sur `tests/profile_e2e.rs`.
- [x] **T7 — `setup`** (AC 6, 7, 9, 10)
  - [x] L'audit entre la création et le commit, dans la transaction ouverte au handler.
  - [x] Tests sur `tests/setup_admin_e2e.rs` : une trace au succès, **aucune** sur les deux `410`
        ni sur les `400` (muets pour deux raisons différentes, cf. AC 6), et **exactement une** sur
        `toctou_race_two_distinct_usernames_creates_exactly_one_admin`.
- [x] **T8 — Un helper d'assertion d'audit partagé** (DRY)
  - [x] `crates/kesh-api/tests/common/mod.rs` n'expose qu'un seul helper et **aucun** pour l'audit :
        chaque fichier de test réécrit son `sqlx::query_scalar`. Cette story en ajoute quatorze —
        c'est le moment, et la règle DRY du projet l'impose.
- [x] **T9 — Le registre des routes mutantes et sa garde** (AC 11)
  - [x] Inscrire **108 routes** — les 105 de `lib.rs` **plus les trois de `test_endpoints.rs`** —,
        chacune `traced` ou `exempt("<justification>")`. ⚠️ *L'ensemble clos de l'inventaire est
        celui de `lib.rs` ; celui du registre est plus large, et c'est voulu.*
  - [x] ⚠️ Les 15 exemptions portent le **numéro de l'issue** qui les suit — **[#434]** pour les
        onze routes d'onboarding, **[#435]** pour les quatre d'`auth`/session. *Une justification
        sans suivi est un abandon déguisé.*
  - [x] ⚠️ Le test `admin_pat_denied_e2e` lit déjà `lib.rs` par `include_str!` et exige un compte
        exact entre les marqueurs `KESH-ADMIN-ROUTES-BEGIN/END` : **s'en inspirer, et ne pas
        déplacer les marqueurs**.
- [x] **T10 — Propagation du symptôme, avant la première passe de revue**
  - [x] Les **huit sites nommés de l'AC 12** — `admin-manual.tex:1762`, `:1782`, `:1786`, `:1935`,
        `:1956`, `:2184`, et `user-manual.tex:1591-1594`, `:1733` — puis **régénérer les trois PDF**
        (`make fr` dans `docs/manual/`) et les commiter.
  - [x] ⛔ **Ne pas croire un balayage sur parole, fût-il le sien.** La passe 1 avait déclaré le
        symptôme « circonscrit au manuel administrateur » : la passe 2 y a trouvé **deux sites de
        plus, dont un dans un autre manuel**. Le grep portait sur `metadata_json`, qui n'est **qu'un
        des trois** symptômes de l'AC 12 — *greper un symptôme n'est pas greper le défaut.*
  - [x] `grep` des autres affirmations : commentaires « pas d'audit », TODO d'audit, `website/`,
        `README.md`, `docs/api-external.md`. ✅ Ceux-là sont propres, et les manuels DE/EN/IT ne
        portent qu'un `README.md`.
  - [x] Contrôler le **PDF aplati** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`), pas seulement
        le `.tex` : un `grep` naïf sur une phrase coupée rend un faux négatif.
  - [x] ⛔ Partir des **fichiers à couvrir**, pas des mots à trouver : sur la 25-1a, le mot-clé trop
        étroit a laissé passer un site **cinq fois**, dont une par la seule **langue** du support.
- [x] **T11 — Gates**
  - [x] ⛔ **Ciblage interdit** si un patch touche `crates/kesh-db/` — gate complet, exception
        `kesh-db` de la § *« Pendant une boucle de revue »*. Cette story touche des repositories.
  - [x] Base de gate remise à zéro **et vérifiée** avant le gate complet, inconditionnellement.
  - [x] E2E au push ; frontend **non touché** par cette story — le vérifier plutôt que le supposer.

## Dev Notes

### Le patron d'appel, tel qu'il est — relevé au sol, pas supposé

**Une seule fonction écrit dans la piste** : `kesh_db::repositories::audit_log::insert_in_tx`
(`crates/kesh-db/src/repositories/audit_log.rs:61`). Aucun trigger, aucun `INSERT INTO audit_log`
ailleurs. Une route est donc tracée **si et seulement si** son chemin atteint cet appel.

**`insert_in_tx` ne commit jamais** (doc-comment `:54-60`) : elle reçoit la transaction de
l'opération auditée et y écrit. C'est le contrat, et il a une conséquence que la 25-1a a
verrouillée par un test — *l'écriture d'audit partage la transaction de l'opération, donc son
échec fait échouer l'opération métier entière.* Pour une **mutation**, c'est le comportement
voulu : une mutation dont la trace n'a pas pu s'écrire ne doit pas être commitée.

⛔ **`actor_label` n'ajoute RIEN à faire.** La 25-1a a ajouté la colonne **sans toucher la surface
d'appel** : elle est remplie par un sous-SELECT dans l'`INSERT` du repository
(`audit_log.rs:79-80`). `NewAuditLogEntry` n'a pas changé. Ne pas chercher à la renseigner.

#### Quel constructeur — le critère n'est PAS le rôle

| Forme | Quand |
|---|---|
| `NewAuditLogEntry::from_current_user(&current_user, …)` | **le défaut** — dès que le handler a un `&CurrentUser` en main (trait `AuditActor`, `crates/kesh-api/src/audit.rs:18`, son impl `:33`) |
| `NewAuditLogEntry::for_actor(user_id, api_key_id, …)` | en dessous du handler (helper, repository) : on fait **descendre** `(user_id, api_key_id)` |
| `NewAuditLogEntry::user(user_id, …)` | seulement si la route est dans `admin_routes` — un jeton d'API n'y passe pas — **ou** s'il n'y a pas de `CurrentUser` du tout (pré-authentification) |

⚠️ **Le critère est « un jeton d'API peut-il atteindre ce chemin », pas « la route est-elle
admin-only ».** `require_not_pat` n'est posé **qu'une fois**, en `route_layer` du bloc
`admin_routes` (`crates/kesh-api/src/lib.rs:330`). Tout ce qui est hors de ce bloc est
atteignable par un jeton personnel ; `::user` y écrirait `actor_type = 'user'` sur une action
faite par une intégration. *C'est exactement la dette [#431], et une story qui la reproduit
l'aggrave.*

⚠️ **Ce que `::user` perd n'est pas l'imputabilité humaine** — `user_id` porte alors le créateur
de la clé — **mais l'information « c'était une intégration, pas une personne ».**

#### Les conventions, relevées sur l'ensemble des libellés d'action du dépôt

⚠️ *Ce relevé portait sur les **libellés** — il se trouve qu'ils étaient 73, comme les routes
tracées. Deux grandeurs sans rapport que le même nombre rendait indiscernables : l'énoncé nomme
désormais son objet plutôt que son total.*

- **Action** : `<entité_singulier>.<participe_passé>` — `contact.created`, `invoice.validated`,
  `journal_entry.reversed`. Les écarts existants (`books.locked`, `admin.full_import`,
  `exports.global`) sont des précédents documentés, **pas des autorisations**.
- **`entity_type`** : le nom de table **au singulier**. Trois exceptions historiques
  (`bank_imports`, `bank_profiles`, `reconciliation_rules`) — ne pas les imiter.
- **`details_json`** : **snake_case**, et c'est écrit (`routes/reports.rs:1350-1367`) — la surface
  HTTP est en camelCase, la piste en snake_case, pour que `details_json->>'$.field_name'` marche
  en SQL.
- **Jamais de secret dans les détails** : le dépôt écrit `"iban_present": true` plutôt que l'IBAN
  (`routes/bank_accounts.rs:458-462`). Aucun mot de passe, hash, jeton ni IBAN complet.
- **`before`/`after` avec `version`** est la forme canonique d'un `updated`
  (`routes/bank_accounts.rs:560-578`).

#### Les tests — deux patrons, et celui-ci n'est pas le léger

| Patron | Quand |
|---|---|
| `audit_log::find_by_entity(&pool, "<type>", id, 10)` puis `.any(\|e\| e.action == "…")` | prouver qu'une action **a été écrite** |
| `sqlx::query_as` direct sur `audit_log` | ⛔ **le patron de cette story** — dès qu'on assertionne l'**attribution** (`actor_type`, `actor_api_key_id`, `user_id`) ou le contenu de `details_json` |

Référence du second : `crates/kesh-api/tests/api_keys_e2e.rs:484-509`, le seul test qui prouve le
comportement par jeton de bout en bout.

⚠️ **`details_json` se lit en `Option<Vec<u8>>`, pas en `Value`** — MariaDB le stocke en blob
binaire, puis `serde_json::from_slice`. Précédent : `tests/reports_e2e.rs:977-1010`, dont le
commentaire dit pourquoi l'assertion porte sur le **contenu** : *« auparavant seuls
user/action/entity_* étaient assertés → faux-vert si une régression renommait les clés JSON »*.

### Où poser l'audit — le fait structurant, et la décision route par route

⛔ **`insert_in_tx` ne prend qu'une `&mut Transaction`.** Or **six** des quatorze chemins passent
par un repository qui **ouvre et commite sa propre transaction en interne** — et **cinq autres
n'ouvrent aucune transaction du tout**, et trois en ont déjà une au handler — **6 + 5 + 3 = 14** : le handler n'a donc
rien où greffer l'audit. C'est le fait qui commande tout le reste, et il n'apparaît nulle part
dans l'issue.

| Route | Transaction aujourd'hui | Décision proposée |
|---|---|---|
| `POST /users` | `users::create` commite seul | **handler** — `create_in_tx` existe déjà (`users.rs:50`) |
| `PUT /users/{id}` | `update_role_and_active` commite seul | **extraire `_in_tx`** — deux routes, deux libellés |
| `PUT /users/{id}/disable` | idem, **même fonction** | **extraire `_in_tx`** — cf. ci-dessus |
| `PUT /users/{id}/reset-password` | `update_password` : **aucune** transaction | **handler** — `update_password_in_tx` existe (`users.rs:320`) |
| `POST /contacts/{id}/persons` | aucune | **extraire `_in_tx`** |
| `PUT /contact-persons/{id}` | aucune | **extraire `_in_tx`** |
| `DELETE /contact-persons/{id}` | aucune | **extraire `_in_tx`** + pré-chargement |
| `POST /inbox-import` | aucune, sur toute la chaîne | **extraire `create_in_tx`** ⚠️ **et** threader l'acteur |
| `POST /…/{id}/discard` | ✅ **déjà ouverte au handler** | rien d'autre à bouger |
| `POST /…/{id}/complete` | ✅ **déjà ouverte au handler** | rien d'autre à bouger — une ligne |
| `PUT /companies/current/email` | `companies::update` commite seul | **extraire `_in_tx`**, enveloppe conservée |
| `PUT /companies/current/contact-details` | idem, **même fonction** | idem |
| `PUT /profile/mode` | `onboarding::update_step` commite seul | **extraire `_in_tx`**, enveloppe conservée |
| `POST /setup/admin` | ✅ **déjà ouverte au handler** | rien d'autre à bouger |

⚠️ **Trois routes sont gratuites** (`discard`, `complete`, `setup/admin`) : la transaction est déjà là, l'appel
se glisse dedans. **Deux autres le sont presque** (`POST /users`, `/reset-password`) : le variant
`_in_tx` existe déjà, écrit pour cette raison même. Commencer par ces quatre donne le patron
complet à moindre risque avant de toucher aux signatures partagées.

⛔ **Pourquoi l'extraction d'un `_in_tx` plutôt qu'un paramètre `user_id` dans le repository** —
c'est l'arbitrage le moins évident de la story. Le dépôt fait les deux (`contacts::create(pool,
user_id, new)` d'un côté, `routes/vat.rs:309-368` de l'autre). Trois raisons penchent ici vers le
variant transactionnel :

1. **`users::update_role_and_active` sert DEUX routes qui doivent écrire deux libellés
   différents.** Passer l'action au repository reviendrait à lui faire porter une décision qui
   n'est pas la sienne.
2. **`companies::update` et `onboarding::update_step` ont d'autres appelants** — leur ajouter un
   paramètre les touche tous, dont l'onboarding et les imports.
3. **Le paramètre `user_id` seul est le mécanisme même de [#431]** : il fait perdre
   `api_key_id` en chemin. Les routes `comptable` de cette story sont atteignables par jeton ;
   descendre `user_id` nu y écrirait un acteur faux.

### Ce que la story ne fait pas

- **`onboarding` (11 routes) → [#434]** et **`auth`/session (4 routes) → [#435]**, ouvertes le
  2026-09-11. Elles sont **exemptées au registre de l'AC 11 en citant leur numéro**, pas oubliées.
  ⚠️ **[#435] pose une question de conception que cette story n'a pas** : tracer chaque `login` et
  chaque `refresh` ferait de la piste un journal de sessions, au risque de noyer les gestes
  comptables. Les quatre routes ne se valent pas.
- **La migration des repositories qui emploient `::user`** → [#431], explicitement hors périmètre.
  ⚠️ **Recensement refait et porté à l'issue le 2026-09-11** : **15 repositories de production et
  38 sites** — l'issue en annonçait 10 —, `audit_log.rs` et ses trois occurrences de `mod tests`
  étant exclus des 38. ⛔ **Le commentaire publié sur GitHub annonçait 35 : son total contredisait
  sa propre ventilation, qui somme à 41.** Corrigé le même jour. *Recompter depuis la source vaut
  aussi pour ce qu'on vient d'écrire ailleurs.* ⚠️ `invoices.rs` porte **les deux formes à la fois** (`::user` ×5 et `for_actor`
  `:2112`) : *aucun décompte par fichier ne montre une migration à moitié faite.*
- **La route et l'écran de consultation** → 25-1c. ✅ **Son arbitrage est rendu** (2026-09-11) :
  `audit_log` prend un `company_id`. ⚠️ **Si ce `company_id` devenait un champ de
  `NewAuditLogEntry` plutôt qu'un sous-SELECT du repository, les quatorze appels ajoutés ici
  devraient le fournir** — à vérifier avant de coder, pas après.
- **Aucun changement de schéma** : la story n'ajoute ni colonne ni migration — donc **ni P2-bis, ni
  P3, ni P5, ni P6, ni P7, ni P8**. ⚠️ *Le vérifier en fin d'implémentation plutôt que de le tenir
  pour acquis : un `_in_tx` extrait ne touche pas le schéma, un index ajouté au passage, si.*
- **Le frontend** : aucune surface visible ne change.

### References

- `audit-experts-2026-08-26.md` § III.3 · `epic-25-vague1-suite.md` § 25-1
- Issue : [#379] · voisines : [#376], [#377] (25-1a, mergée en PR #433), [#378] (25-1c), [#431],
  et les deux ouvertes par cette spécification : [#434] (onboarding), [#435] (`auth`/session)
- Story sœur : `25-1a-piste-inalterable.md` — § *Le patron d'appel* y trouve son état antérieur,
  et sa boucle de revue documente **cinq** échecs du grep de propagation sur une seule story
- Socle : `crates/kesh-db/src/entities/audit_log.rs` (constructeurs `:157`, `:179`, `:203`,
  `AUDIT_ENTITY_ID_NONE` `:97`), `crates/kesh-db/src/repositories/audit_log.rs:61` (`insert_in_tx`,
  contrat `:54-60`, sous-SELECT d'`actor_label` `:79-80`), `crates/kesh-api/src/audit.rs:33`
  (`from_current_user`)
- Blocs de routeur : `crates/kesh-api/src/lib.rs:182-332` (`admin_routes`, marqueurs et compte
  exact), `:330` (`require_not_pat`, **l'unique**), `:336-660` (`comptable_routes`), `:663-943`
  (`authenticated_routes`), `:965` (`setup`, publique)
- Patrons cités : `routes/bank_accounts.rs:741` (garde no-op), `:460-461` (présence et non valeur),
  `routes/companies.rs:155-163` (l'effacement par omission), `repositories/companies.rs:387-390`
  (deux libellés plutôt qu'un), `routes/auth.rs:948-957` (`details_json = None`),
  `auth/bootstrap.rs:252-265` (acteur et cible confondus), `routes/reports.rs:1350-1367`
  (snake_case)
- Tests : `tests/api_keys_e2e.rs:484-509` (attribution par jeton), `tests/reports_e2e.rs:977-1010`
  (`details_json` en `Option<Vec<u8>>`), `tests/period_lock_e2e.rs:546-554` (séquence d'actions),
  `tests/bank_accounts_e2e.rs:582` (pas de doublon en no-op)

## Dev Agent Record

### ⛔ T1 — Décision de conception, arrêtée AVANT de coder

**Recensement des appelants, fait et non supposé** (`grep -rn` sur `crates/`) :

| Fonction | Appelants de production | Tests | ⇒ Geste |
|---|---|---|---|
| `users::create` | **2** — `routes/users.rs:212`, `auth/bootstrap.rs:123` | **~60** | `create_in_tx` **existe déjà** : les deux sites de production y passent, `create` reste intacte pour les tests |
| `users::update_role_and_active` | 2 routes (`:263`, `:314`) | quelques-uns | **extraire `_in_tx`**, enveloppe conservée |
| `users::update_password` | 1 route | — | `update_password_in_tx` **existe déjà** |
| `companies::update` | 2 routes + **`kesh-seed/src/lib.rs:122`** | **9** | **extraire `_in_tx`**, enveloppe conservée — *le seed et les neuf tests ne bougent pas* |
| `onboarding::update_step` | `profile.rs:36` + **8 routes d'onboarding** + `kesh-seed:222` | 6 | **extraire `_in_tx`**, enveloppe conservée et **NON auditée** |
| `contact_persons::{create,update,archive}` | **1 chacune**, et **aucun test** | 0 | **conversion directe en `_in_tx`**, sans enveloppe — *une enveloppe que personne n'appelle est du code mort qui paraît vivant* |
| `imported_supplier_invoices::create` | 1 (`inbox_import.rs:490`) | 1 | **extraire `create_in_tx`** |
| `imported_supplier_invoices::reactivate_to_complete` | 1 (`inbox_import.rs:424`) | — | **extraire `_in_tx`** |

**La décision proposée aux Dev Notes est confirmée sur les huit lignes**, avec deux précisions que
le recensement impose :

1. ⛔ **`users::create` n'est PAS convertie** — soixante tests l'appellent. Seuls ses **deux**
   appelants de production passent à `create_in_tx`, qui existe déjà. *Convertir une fonction que
   soixante tests appellent pour en servir deux serait payer un refactor au prix d'un risque.*
2. ⛔ **`contact_persons` est le seul cas de conversion SANS enveloppe** : une fonction par route,
   aucun test, aucun autre appelant. Partout ailleurs l'enveloppe protège des appelants réels —
   ici elle ne protégerait rien.

### Agent Model Used

Claude Opus 5 (1M context) — implémentation.

### Completion Notes List

**Les quatorze routes sont tracées**, chacune dans la transaction de la mutation
qu'elle décrit. Le geste diffère selon ce que le recensement montrait :

- **quatre extractions de variant `_in_tx`, enveloppe conservée** —
  `users::update_role_and_active`, `companies::update`, `onboarding::update_step`
  et `imported_supplier_invoices::create` (60, 9, 10 et 1 appelants) ;
- **quatre conversions directes, SANS enveloppe** — les trois de
  `contact_persons` et `reactivate_to_complete`, qui n'avaient qu'un appelant
  chacune et aucun test. *Une enveloppe que personne n'appelle est du code mort
  qui paraît vivant.*

⚠️ *La formulation « six extractions » du premier jet mélangeait les deux gestes ;
corrigée en passe 1 de revue de code.*
L'acteur descend le long de `run_inbox_import` → `process_inbox` → `process_one_file`.

⛔ **CINQ DÉFAUTS QUE SEULE L'EXÉCUTION A RÉVÉLÉS, et ce sont tous des détecteurs
mal formés — les miens.** Aucune des sept passes de revue de spécification ne
pouvait les voir :

1. **`creditor_iban` n'est pas `Option<String>`** — le compilateur a rejeté le
   `is_some()` que la spec prescrivait. ⇒ `!is_empty()`.
2. ⛔ **Les quatre helpers d'audit cassaient le gate.** `tests/common/mod.rs` est
   compilé **séparément dans chaque binaire de test** : un helper employé par dix
   fichiers sur quarante est « jamais utilisé » dans les trente autres, et
   `clippy -D warnings` refusait. ⇒ `#![allow(dead_code)]`, motivé en tête du
   module.
3. ⛔ **Mon extracteur du registre rendait un ensemble VIDE** sur
   `test_endpoints.rs`, dont la forme diffère (`.route("/x", post(h))` et non
   `post(routes::m::h)`) — et **comparait donc deux ensembles vides avec
   succès**. ⇒ garde explicite contre l'ensemble vide.
4. ⚠️ **Mon contrôle de PDF a rendu un faux négatif** sur une apostrophe
   **typographique** cherchée en apostrophe droite. Le piège que le `CLAUDE.md`
   documente, pris sur le fait.
5. ⛔ **Ma remise à zéro de la base de gate était INCOMPLÈTE, puis OMISE.** La
   première fois le **seed** a été sauté (29 tests tombés sur *« need at least
   one Admin user in DB »*) ; la seconde, la base n'a pas été reconstruite entre
   deux runs alors que la règle est **inconditionnelle**. ⚠️ Et la première
   erreur a été masquée par une **vérification mal formée** : elle comptait les
   migrations et les tables — ce qui était commode — au lieu de ce dont les tests
   ont besoin. *Vérifier n'est pas exécuter ; mais vérifier la mauvaise chose ne
   vaut pas mieux que ne rien vérifier.*

**Un test a rougi à raison, et c'était le TEST** : il comptait les traces
globalement alors que le chemin d'amorçage écrit désormais la sienne. Les trois
compteurs globaux du dépôt ont été inspectés ensuite — deux robustes par
comparaison, un passant par l'enveloppe non auditée. Aucun cassé.

**Deux gardes éprouvées par MUTATION** : la garde no-op de `users` (neutralisée
→ rouge, rétablie → vert) et celle du registre (une entrée retirée → elle nomme
la route manquante).

**Gates** — base détruite, reconstruite **par les trois étapes** et vérifiée sur
ce dont les tests ont besoin (1 admin, zéro résidu) :

| Gate | Résultat |
|---|---|
| `cargo fmt --all -- --check` | propre |
| `cargo clippy --workspace --all-targets -- -D warnings` | propre |
| `scripts/test-fast.sh --ci` | **2318/2318, 4 skipped, 90 s** |
| frontend | **non touché** — vérifié par `git diff --stat`, pas supposé |
| i18n | **non touché** — aucune clé ajoutée |

⚠️ **E2E au push**, conformément à la règle.

**Décompte recoupé** : 2307 (25-1a) + **11 tests neufs** = 2318 — deux sur
`users`, un sur `profile`, quatre sur `contact_persons` *(fichier créé)*, quatre
sur le registre. Les autres assertions sont greffées sur des tests existants.

### File List

**Backend — persistance** *(cinq repositories)*
- `crates/kesh-db/src/repositories/users.rs` *(extraction `update_role_and_active_in_tx`)*
- `crates/kesh-db/src/repositories/companies.rs` *(extraction `update_in_tx`)*
- `crates/kesh-db/src/repositories/onboarding.rs` *(extraction `update_step_in_tx`, **non auditée**)*
- `crates/kesh-db/src/repositories/contact_persons.rs` *(conversion, en-tête corrigé)*
- `crates/kesh-db/src/repositories/imported_supplier_invoices.rs` *(extractions `create_in_tx`, `find_by_id_scoped_in_tx`, conversion de `reactivate_to_complete`)*

**Backend — API**
- `crates/kesh-api/src/routes/users.rs` · `companies.rs` · `contact_persons.rs` ·
  `profile.rs` · `setup.rs` · `imported_supplier_invoices.rs`
- `crates/kesh-api/src/auth/bootstrap.rs` *(le second chemin de création du premier admin)*
- `crates/kesh-api/src/inbox_import.rs` *(threading de l'acteur, transaction par pièce)*

**Tests**
- `crates/kesh-api/tests/common/mod.rs` *(quatre helpers d'audit)*
- `crates/kesh-api/tests/contact_persons_e2e.rs` *(**nouveau** — ces routes n'avaient aucun test)*
- `crates/kesh-api/tests/audit_route_registry.rs` *(**nouveau** — le registre et sa garde)*
- `users_e2e.rs` · `profile_e2e.rs` · `companies_e2e.rs` · `setup_admin_e2e.rs` ·
  `inbox_import_e2e.rs`

**Documentation**
- `docs/manual/fr/admin-manual.{tex,pdf}` *(six sites)*
- `docs/manual/fr/user-manual.{tex,pdf}` *(deux sites, dont un renvoi mort)*
- `docs/manual/fr/marketing-brochure.pdf` *(régénéré par `make fr`)*

### Debug Log References

### Completion Notes List

### File List

## Change Log

### Passe 1 de `bmad-create-story validate` — deux lentilles, contexte frais

**Sonnet 4.6 · Haiku 4.5**, orthogonales à l'auteur (Opus 5), prompt versionné
(`25-1b-validate-prompt-p1.md`). **1 CRITICAL, 1 HIGH, 3 MEDIUM, 2 LOW** — tous
vérifiés au sol avant traitement, tous patchés.

| # | Sév. | Origine | Objet |
|---|---|---|---|
| P1-1 | **CRITICAL** | Sonnet | `imported_supplier_invoices::create` écrit **sur le pool** et n'a **aucun `create_in_tx`** ⇒ l'**AC 9 était intenable sur cette seule route** |
| P1-2 | **HIGH** | orchestrateur | `admin-manual.tex:1782` promet que **toute** action mutante est tracée — faux pour 28 routes, et **toujours faux après cette story** ⇒ **AC 12** |
| P1-3 | MEDIUM | Sonnet | AC 11 : comparer un **nombre** ne détecte pas un échange de routes ⇒ **diff ensembliste par identité** |
| P1-4 | MEDIUM | Haiku, **étendu** | manuel : **trois** champs faux sur sept, non un seul |
| P1-5 | MEDIUM | Sonnet · Haiku *(convergence)* | `admin-manual.tex:1956` : la réserve reste vraie, son **exemple** devient faux |
| P1-6 | LOW | Sonnet | `companies.rs` — le `if` no-op est à `:181`, hors de la plage citée |
| P1-7 | LOW | Sonnet | `audit.rs` — le trait est à `:18`, son impl à `:33` |
| — | *écarté* | Haiku | `contact_persons.rs:5` était **déjà nommé par l'AC 2** |

### Ce que cette passe apprend

1. **Le CRITICAL est un invariant qui se contredit lui-même sur une seule route.**
   L'AC 9 affirmait valoir « pour les treize » ; il en restait une où il était
   matériellement impossible. *Un invariant énoncé sans avoir été vérifié route
   par route est une déclaration, pas une garantie.*
2. **Vérifier un finding rapporte plus que le finding.** La lentille signalait
   **un** champ faux au manuel ; la vérification en a trouvé **trois**, plus
   l'affirmation qui ouvre la section et qu'aucune lentille n'avait lue — la plus
   forte du passage. *C'est le motif de la passe 2 de la 25-1a, dans l'autre
   sens : là un faux positif avait révélé un trou, ici un vrai positif sous-évalué
   en cachait un plus gros.*
3. **Un détecteur mal formé coûte le même diagnostic qu'un défaut réel.** L'AC 11
   comptait là où il fallait comparer des identités. Le dépôt avait déjà écrit la
   leçon pour les préfixes de KF ; elle vaut pour les registres de routes.
4. ⚠️ **Trouvé en chemin, hors périmètre** : `user-manual.tex:503` nie un verrou
   de période que `:444-453` documente — le manuel se contredit à cinquante lignes
   d'écart depuis la 24-4c. **À tracer.**

### Passe 2 de `bmad-create-story validate` — lentille unique (Opus 5), contexte frais

Prompt versionné (`25-1b-validate-prompt-p2.md`), braqué en priorité sur les sept
patches de la passe 1. **0 CRITICAL, 3 HIGH, 5 MEDIUM, 6 LOW** — sévérité en
**recul**, donc pas de split déclenché.

⚠️ **Ventilation par origine : 13 findings sur 15 sont D'ORIGINE**, un seul HIGH et
un seul LOW naissant d'un patch. *C'est l'inverse du motif habituel de ce dépôt —
la remédiation n'a presque rien cassé, c'est la conception qui était incomplète.*

| # | Sév. | Origine | Objet |
|---|---|---|---|
| P2-1 | **HIGH** | d'origine | **Un TROISIÈME chemin d'entrée** : la **réactivation** d'une pièce écartée rend `Accepted` sans rien créer ⇒ la piste dirait `discarded` sur une pièce `to_complete` |
| P2-2 | **HIGH** | d'origine | *« son chemin jumeau l'est »* est **faux** : `auth/bootstrap.rs:123` crée le même premier administrateur et **n'écrit rien** — et ce n'est pas une route, donc écrit **nulle part** |
| P2-3 | **HIGH** | **du patch P1** | Le balayage déclaré « déjà fait » était faux : **deux sites de plus**, dont un dans un **autre manuel** |
| P2-4 | MEDIUM | d'origine | AC 8 : **quatre** routes court-circuitent le no-op, pas trois |
| P2-5 | MEDIUM | d'origine | [#431] : **38** sites, pas 35 — le total publié contredisait sa ventilation |
| P2-6 | MEDIUM | d'origine | AC 6 : deux `410` et un `500` après le `begin`, quatre `400` **avant** — deux silences, deux mécanismes |
| P2-7 | MEDIUM | d'origine | AC 11 : l'ensemble clos est celui de `lib.rs`, **pas du backend** — trois routes vivent dans `test_endpoints.rs` |
| P2-8 | MEDIUM | **du patch P1** | *« transaction portée par `process_one_file` »* ne dit pas **où** : pool à **5 connexions**, à poser autour de la seule insertion |
| P2-9..14 | LOW | d'origine | issue manquante, référence divergente, « 170 » pour 174, traçabilité AC↔tâches, révocation hors transaction, coexistence de deux libellés |

### Ce que cette passe apprend

1. ⛔ **Déclarer un balayage complet est une affirmation comme une autre, et elle
   se vérifie.** La passe 1 avait écrit « symptôme circonscrit au manuel
   administrateur » ; il restait deux sites, dont un dans un autre manuel. Le grep
   portait sur `metadata_json` — **un** des trois symptômes de l'AC 12. *Greper un
   symptôme n'est pas greper le défaut.* C'est la **sixième** fois que ce geste est
   pris en défaut sur ces deux stories.
2. **Le CRITICAL de la passe 1 avait un jumeau à soixante lignes.** Corriger
   `create` sans regarder le `match` qui le précède laissait `reactivate_to_complete`
   intact — *un patch qui ne remonte pas au site voisin ferme la moitié d'un trou.*
3. **Un total publié ailleurs se recompte aussi.** Le commentaire porté à [#431]
   annonçait 35 sites quand sa propre ventilation sommait à 41 : les trois
   occurrences de test avaient été soustraites deux fois. Rectifié sur l'issue.

### Passe 3 de `bmad-create-story validate` — lentille unique (Sonnet 4.6), contexte frais

Prompt versionné (`25-1b-validate-prompt-p3.md`), **deux fronts d'égal poids** : les patches de la
passe 2, et **les axes que la passe 2 avait déclaré ne pas avoir exercés**. **0 CRITICAL, 1 HIGH,
2 MEDIUM, 2 LOW** — troisième recul consécutif, et **les cinq sont d'origine**.

| # | Sév. | Objet |
|---|---|---|
| P3-1 | **HIGH** | **T6 contredisait l'AC 9 en silence** : elle ne disait ni d'extraire `update_step_in_tx`, ni qui mène la transaction — et **placer la trace dans `update_step` ferait écrire `installation.ui_mode_changed` à chaque étape de l'onboarding** |
| P3-2 | MEDIUM | **Un SIXIÈME site du manuel**, et il parle d'**attribution** et non de couverture : `admin:1762` promet `actor_type = 'api_key'` sur les mutations par clé — faux pour les 38 sites de [#431] |
| P3-3 | MEDIUM | « 1 partielle » n'avait pas de **portée déclarée** : [#434] en documente une seconde, et le chiffre a été relu contre le mauvais périmètre |
| P3-4 | LOW | La branche `UniqueConstraintViolation` du chemin d'amorçage change de forme |
| P3-5 | LOW | Les trois routes de `test_endpoints.rs` doivent sortir du **diff automatique** — un garde qui rougit toujours est un garde qu'on désactive |

### Ce que cette passe apprend

1. ⛔ **Reprendre les axes qu'une passe déclare n'avoir pas exercés paie deux fois.** La règle du
   dépôt disait « c'est exactement là que se trouve ce qu'elle a manqué » : c'est vrai — l'axe
   « contenu de [#434] » a rendu P3-3, l'axe « faisabilité » a rendu P3-1. *Une passe qui déclare
   honnêtement ses manques rend la suivante plus efficace que si elle avait bluffé.*
2. ⛔ **Un inventaire construit sur une seule question ne trouve que les réponses à cette
   question.** Cinq relectures du manuel cherchaient « ce qui est tracé » ; le sixième site dit
   « comment c'est attribué ». Il était là depuis le début, en évidence.
3. **Une trace au mauvais étage ne manque pas : elle ment.** Le HIGH n'est pas un oubli d'écriture,
   c'est un risque de trace **surnuméraire et fausse** sur neuf chemins hors périmètre.

### Passe 4 de `bmad-create-story validate` — lentille unique (Haiku 4.5), contexte frais

Prompt versionné (`25-1b-validate-prompt-p4.md`), axe principal : **les 73 routes réputées
tracées**, que trois passes avaient déclaré ne pas avoir exercées.

⛔ **La lentille a rendu 0 finding en déclarant n'avoir traité que 4 routes sur 73**, ni le
septième site du manuel, ni les recomptes. **Elle ne compte donc pas comme passe** — mais sa
franchise a fait son travail : *un « 0 finding » adossé à la liste de ce qui n'a pas été fait vaut
infiniment mieux qu'un « 0 finding » qui se tait.* L'orchestrateur a repris les trois axes.

**L'axe principal, repris et exercé autrement** — non route par route, mais par les **94 appels
d'audit** du backend et leur gouvernance : **8 sont sous condition**, et les huit sont justifiés
(garde no-op, lot vide, verrou qui recule, best-effort, complémentarité). ⇒ **Aucun trou de
couverture nouveau.**

⚠️ **Ce que cette méthode ne voit pas, et il faut l'écrire** : une route dont un chemin de succès
**contournerait** le repository qui trace. Elle établit qu'aucune trace existante n'est
conditionnée à tort ; elle n'établit pas que tout chemin en atteint une.

| # | Sév. | Objet |
|---|---|---|
| P4-1 | MEDIUM | **Un SEPTIÈME site de manuel** : `admin:1935` gage la conformité **OLICo al. 3** sur un ZIP d'export censé contenir `audit_log` — il ne le contient pas ([#386]) |
| — | *hors périmètre* | `reconciliation_rule.deleted` écrit son attribution **différemment selon l'effet de l'opération** ⇒ porté à [#431] |
| — | *hors périmètre* | Le site `:1935` porté à [#386], avec la correction inverse à prévoir quand l'export intégrera la piste |

### Ce que cette passe apprend

1. ⛔ **Sept sites de manuel, sept questions différentes.** Couverture, champs, attribution,
   exportabilité : *chaque site a été trouvé en posant une question que le précédent ne posait
   pas.* Un inventaire documentaire ne se clôt pas par l'épuisement des fichiers, mais par celui
   des **questions** — et rien ne dit qu'il n'en reste pas une.
2. **Une passe peut échouer utilement.** Celle-ci n'a pas fait son axe ; en le disant, elle a
   désigné exactement où chercher. *C'est le contraire du précédent de la 24-5, où une passe avait
   qualifié de « robuste » un point non exécuté.*
3. **L'incohérence vaut pire que l'absence.** `reconciliation_rule.deleted` est toujours tracée,
   mais son `actor_type` dépend de l'effet de l'opération : un réviseur qui filtre sur les jetons
   verra les suppressions **sans effet** et manquera celles qui en ont eu un.

### Passe 5 CIBLÉE — lentille unique (Opus 5), contexte frais

Prompt versionné (`25-1b-validate-prompt-p5.md`), périmètre **la remédiation des passes 3 et 4**,
non la story. **0 CRITICAL, 0 HIGH, 4 MEDIUM, 5 LOW.**

| # | Sév. | Origine | Objet |
|---|---|---|---|
| P5-1 | MEDIUM | **du patch P3** | **Le patch a annulé le bénéfice qu'il invoquait** : sortir les trois routes de `test_endpoints.rs` du diff entier rend une **quatrième** route de ce fichier aussi invisible qu'avant ⇒ exclusion du **second volet seulement** |
| P5-2 | MEDIUM | **du patch P2** | **La propagation 13 → 14 était incomplète** : le commit annonçait « six énoncés », il y en avait **huit** — l'AC 7 n'assignait de constructeur qu'à treize routes, T4 s'intitulait « deux routes » pour trois |
| P5-3 | MEDIUM | d'origine | **Un HUITIÈME site de manuel** : `user-manual:1733`, le glossaire — même promesse que `:1591-1594`, plus un **renvoi vers une section qui n'existe pas** |
| P5-4 | MEDIUM | **du patch P2** | L'AC 12 s'intitulait *« le manuel administrateur… trois affirmations »* quand sa table en listait sept dans **deux** manuels |
| P5-5..9 | LOW | mixte | « sept des quatorze chemins » (il y en a **six**), une ligne vide coupant la table, deux puces de T6 contradictoires, T7 n'appelant pas l'AC 7, deux aphorismes collés |

### Ce que cette passe apprend

1. ⛔ **Un inventaire ne se clôt ni par l'épuisement des FICHIERS, ni par celui des QUESTIONS, mais
   par leur PRODUIT.** Les sept premiers sites avaient chacun demandé une question neuve ; le
   huitième répond à une question **déjà posée**, dans un fichier qu'elle n'avait pas visité.
   *La leçon de la passe 4 était vraie et incomplète — elle orientait la recherche vers des thèmes
   neufs et détournait du balayage exhaustif.*
2. ⛔ **Un patch peut laisser debout la justification qu'il vient de vider.** P5-1 : la phrase
   « sans quoi une quatrième route ne ferait rougir aucun garde » a survécu au patch qui la rendait
   fausse. *Corriger un mécanisme sans relire ce qui le motivait produit un texte qui se réfute
   lui-même.*
3. ⛔ **« Propagé partout » est une affirmation, donc elle se vérifie.** Mon commit disait six
   énoncés ; il y en avait huit — et c'est exactement le défaut que la passe 2 avait reproché à la
   spec. *Celui qui corrige une dérive de compteur n'est pas immunisé contre elle.*

### Passe 6 CIBLÉE — lentille unique (Sonnet 4.6), contexte frais

Prompt versionné (`25-1b-validate-prompt-p6.md`), périmètre **le seul commit `2f87b1aa`**.
**0 CRITICAL, 0 HIGH, 2 MEDIUM, 0 LOW** — et **les deux sont nés du correctif de la passe 5**.

| # | Sév. | Objet |
|---|---|---|
| P6-1 | MEDIUM | *« quatre autres n'ouvrent aucune transaction »* — le tableau juste en dessous en liste **cinq** ; 6 + 5 + 3 = 14 |
| P6-2 | MEDIUM | **T10 listait encore SEPT sites** quand l'AC 12 en comptait huit : *le patch avait mis à jour le critère et pas la tâche qui l'exécute* |

✅ **Deux vérifications demandées, toutes deux favorables** : le constructeur de `complete` est
**exact** — la route est dans `comptable_routes`, donc atteignable par un jeton, donc
`from_current_user` —, et **aucun neuvième site** n'existe : le manuel utilisateur ne fait
simplement aucune promesse sur les champs, l'attribution ou l'exportabilité.

⛔ **Trois corrections appliquées, plus un balayage que la passe n'avait pas demandé.** Le motif
était devenu clair — *la passe 5 corrige, la passe 6 trouve deux défauts du patch de la passe 5* —
et y répondre par une passe de plus l'aurait reconduit. J'ai donc appliqué la § *Propagation
post-patch* moi-même, sur **tous** les énoncés numériques du document. Elle a rendu :

- l'ambiguïté **105 contre 108** entrées au registre, que la passe 6 signalait sans la trancher :
  l'inventaire porte sur `lib.rs` (105), **le registre est plus large** (108) — et c'est voulu ;
- ⛔ **un dernier piège que six passes ont laissé passer** : *« les conventions, vérifiées sur les
  **73 sites** en place »*. Ce 73 désignait les **libellés d'action**, pas les 73 routes tracées.
  **Deux grandeurs sans rapport que le même nombre rendait indiscernables.** L'énoncé nomme
  désormais son objet plutôt que son total.

### Ce que cette passe apprend

1. ⛔ **Un patch met à jour le critère et oublie la tâche qui l'exécute.** P6-2 : l'AC 12 disait
   huit, T10 disait sept. *Un développeur suivant la tâche à la lettre aurait corrigé sept sites
   sur huit — et le gate serait resté vert.*
2. ⛔ **Deux grandeurs différentes portant le même nombre sont indétectables à la relecture.**
   Seul un balayage qui interroge **ce que chaque nombre compte**, et non sa valeur, pouvait le
   voir. *C'est la § « greper la VALEUR, pas la formulation » retournée : ici la valeur était
   juste et l'objet faux.*
3. **Quand une passe ne trouve plus que des défauts de son propre patch, ajouter une passe
   reconduit le motif.** Ce qu'il faut ajouter, c'est le **geste** — la propagation systématique —
   et le faire soi-même.

### Passe 7 CIBLÉE — lentille unique (Haiku 4.5), contexte frais

Prompt versionné (`25-1b-validate-prompt-p7.md`), périmètre **le seul commit `1f02097f`**.
**0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW** — quatre axes exercés, axes non exercés déclarés et tous
**logiquement impossibles à ce stade** (vérifier des PDF régénérés ou un registre écrit, pour une
spec non implémentée).

**Les deux affirmations principales ont été reprises par l'orchestrateur** — *un « 0 finding » se
vérifie comme un finding* : plus aucun « sept sites » hors Change Log, et la somme du tableau des
décisions fait bien **6 + 5 + 3 = 14**.

⚠️ **Et le contrôle a produit un faux positif instructif** : un test cherchant le mot « audit » sur
la ligne `admin:1786` l'a signalée comme vide, alors qu'elle énumère des **noms de colonnes** et ne
contient pas le mot. *Un détecteur mal formé coûte le même diagnostic qu'un défaut réel* — jusque
dans la vérification d'une clôture.

---

## ✅ Boucle de validation CLOSE — sept passes

| Passe | Modèle(s) | Rendu | Ce qu'elle a trouvé |
|---|---|---|---|
| 1 | Sonnet 4.6 · Haiku 4.5 | **1 C**, 1 H, 3 M, 2 L | un invariant **intenable sur une seule route** |
| 2 | Opus 5 | 0 C, **3 H**, 5 M, 6 L | un **troisième chemin d'entrée**, une justification fondée sur un fait inexistant |
| 3 | Sonnet 4.6 | 0 C, **1 H**, 2 M, 2 L | une trace qui, posée au mauvais étage, **mentirait sur neuf chemins** |
| 4 | Haiku 4.5 *(+ reprise)* | **1 M** | un septième site de manuel, et **l'axe principal non exercé — déclaré** |
| 5 | Opus 5 *(ciblée)* | 0 C, 0 H, **4 M**, 5 L | un patch ayant **vidé la justification qu'il laissait debout** |
| 6 | Sonnet 4.6 *(ciblée)* | 0 C, 0 H, **2 M** | un critère mis à jour **sans la tâche qui l'exécute** |
| 7 | Haiku 4.5 *(ciblée)* | **0** | rien — **critère d'arrêt atteint** |

**Sévérité maximale, passe après passe : CRITICAL → HIGH → HIGH → MEDIUM → MEDIUM → MEDIUM → rien.**
Décroissance stricte. Le rebond du **nombre** de MEDIUM en passe 5 a été porté au Project Lead au
titre de la § *Règle de splitting préventif* ; arbitrage rendu : **pas de split**, aucun de ces
findings ne touchant la conception.

### Ce que cette boucle laisse au-delà de la story

1. ⛔ **Un inventaire ne se clôt ni par l'épuisement des FICHIERS, ni par celui des QUESTIONS, mais
   par leur PRODUIT.** Huit sites de manuel, trouvés en quatre questions × deux fichiers — et le
   huitième par une question déjà posée dans un fichier qu'elle n'avait pas visité.
2. ⛔ **Deux grandeurs différentes portant le même nombre sont indétectables à la relecture.**
   « 73 » désignait tantôt les routes tracées, tantôt les libellés d'action. Seul un balayage
   interrogeant *ce que chaque nombre compte* — et non sa valeur — pouvait le voir.
3. ⛔ **Un patch met à jour le critère et oublie la tâche qui l'exécute.** Le développeur aurait
   corrigé sept sites sur huit, **et le gate serait resté vert**.
4. ⛔ **Quand une passe ne trouve plus que les défauts de son propre patch, ajouter une passe
   reconduit le motif** — ce qu'il faut ajouter, c'est le **geste**, et le faire soi-même.
5. ✅ **Une passe qui déclare honnêtement ce qu'elle n'a pas fait vaut mieux qu'une passe qui
   bluffe** : celle de la passe 4 a désigné exactement où chercher, et deux findings de la passe 3
   en sont nés.

**Statut : `ready-for-dev`.** Prochaine étape : `bmad-dev-story`.

---

## Boucle de revue de code

### Passe 1 — trois lentilles, contexte frais

Prompt versionné (`25-1b-review-prompt-p1.md`), diff **aplati**, modèles orthogonaux à l'auteur
(Sonnet 4.6 ×2 · Haiku 4.5). **0 CRITICAL, 4 HIGH, 1 MEDIUM, 2 LOW.**

| # | Sév. | Lentille | Objet |
|---|---|---|---|
| P1-1 | **HIGH** | attribution/tests | ⛔ **L'extracteur du registre laissait passer un ALIAS** verbe+handler — vérifié **empiriquement** : une route réutilisant un handler déjà inscrit laissait le test **vert**. C'était le faux vert que l'AC 11 nommait |
| P1-2 | **HIGH** | attribution/tests | ⛔ **Trois des quatre actions d'import n'étaient pas testées**, dont **`.reactivated`** — le chemin du défaut le plus sévère de toute la boucle de validation |
| P1-3 | **HIGH** | attribution/tests | T7 revendiquait des tests sur les refus de `setup` et « exactement une trace » sur la race TOCTOU : **aucun n'existait** |
| P1-4 | **HIGH** | attribution/tests | T5 revendiquait qu'un test prouve qu'un refus n'écrit rien : **il ne contenait aucune assertion d'audit** |
| P1-5 | MEDIUM | atomicité | Orphelinat de fichier possible si l'audit échoue après archivage ⇒ **documenté**, non corrigé |
| P1-6 | LOW | atomicité | « six extractions » mélangeait extractions et conversions |
| P1-7 | LOW | orchestrateur | Le grep de `Pas d'audit log` rend la **note historique** qui documente la correction — faux positif à trier à la main |

✅ **Ce que les lentilles ont confirmé, preuves à l'appui** : l'atomicité aux quinze sites
(quatorze routes + l'amorçage), l'attribution **15/15**, l'absence de secret **15/15**, les
`rollback` retirés des corps extraits bien assurés par le `Drop` de sqlx chez les appelants, la
transaction d'import ouverte **autour de la seule insertion** (décodage QR et archivage disque en
dehors, au plus 2 connexions sur 5), les douze critères tenus et les huit sites de manuel exacts —
**aucun neuvième**.

### ⛔ Ce que cette passe apprend, et c'est la leçon la plus coûteuse de la story

1. ⛔ **TROIS des quatre HIGH portent sur des couvertures de test que la SPEC revendiquait et que
   je n'avais pas écrites.** C'est exactement la faute que le `CLAUDE.md` nomme — *le story file
   ne doit affirmer que ce qui a tourné* — commise en rédigeant les tâches **avant** de coder,
   puis jamais relue contre le code livré. ⚠️ *Une tâche qui décrit un test est une promesse ; la
   cocher sans l'avoir écrit la transforme en mensonge, et le gate reste vert.*
2. ⛔ **QUATRE fois dans cette seule remédiation, mes assertions de « rien n'est écrit » étaient
   MAL FORMÉES** : elles mesuraient un **total absolu** là où il fallait l'**écart autour du
   refus**, le montage du test ayant légitimement tracé. Chacune a rougi et chacune a été
   corrigée — mais quatre fois, c'est un motif, non un accident. *Prouver qu'un refus n'écrit rien
   exige de mesurer un DELTA ; un absolu mesure autre chose que ce qu'il prétend.*
3. ✅ **Deux gardes de plus éprouvées par mutation** : celle des alias (route dupliquée → rouge,
   retirée → vert) et, déjà, celle du no-op. *Une garde qu'on n'a pas vue rougir ne garde rien.*

**Gate après remédiation** : base reconstruite par les **trois** étapes et vérifiée,
`fmt` propre, `clippy --workspace --all-targets -D warnings` propre,
**2318/2318** (4 skipped, 89 s) — nombre inchangé, les assertions ayant été greffées sur des
tests existants.

### Passe 2 — lentille unique (Opus 5), contexte frais

Prompt versionné (`25-1b-review-prompt-p2.md`), braquée sur la remédiation de la passe 1.
**0 CRITICAL, 1 HIGH, 4 MEDIUM, 4 LOW** — dont **trois nés de cette remédiation**.

| # | Sév. | Origine | Objet |
|---|---|---|---|
| P2-1 | **HIGH** | d'origine *(sa clôture déclarée naît de la P1)* | ⛔ **La garde contre les alias DÉPLAÇAIT le faux vert au lieu de le fermer** — prouvé par exécution : une route écrite `post(users::create_widget)`, sans le préfixe `routes::`, n'était **pas extraite**, et le test restait vert |
| P2-2 | MEDIUM | d'origine | Rien n'exigeait qu'un **troisième fichier de routes** soit couvert |
| P2-3 | MEDIUM | **de la P1** | Le commentaire du 410 attribuait le silence au **`rollback`** : ce chemin est un `if` **avant** l'INSERT — faute commise dans le fichier même où le test voisin l'interdit |
| P2-4 | MEDIUM | **de la P1** | Mon patch « de propagation » avait **AFFAIBLI** la seule assertion qui établissait son propre énoncé |
| P2-5 | MEDIUM | d'origine | ⛔ **DÉFAUT DE PRODUCTION** : `PUT /users/{id}` **efface l'e-mail par omission** et la trace ne le disait pas |
| P2-6..9 | LOW | mixte | dénominateur du manuel, quatre copies d'une requête, commentaire de troncature, trace non reliée au gagnant de la race |

### ⛔ Ce que cette passe apprend

1. ⛔ **Fermer un faux vert par l'énumération d'une forme qui marche ne le ferme pas — il le
   déplace.** La garde de la passe 1 traitait les **alias** ; la forme d'écriture suivante passait.
   La § *Inventorier les sites NON RÉSOLUS* du `CLAUDE.md` dit quoi faire, et je ne l'avais pas
   appliquée à mon propre détecteur. ⇒ **l'inventaire est désormais vide**, et la cause réelle —
   une troncature de source tombant à la **fin** du bloc commenté au lieu de son **début** — est
   corrigée. *Éprouvé par la mutation même qui passait inaperçue.*
2. ⛔ **Propager un remède hors de son domaine est une régression.** Mon patch proactif a filtré
   par action une assertion dont l'absolu était **exact** (`truncate_all` en tête) et la forme la
   **plus forte**. Le message de commit diagnostiquait « il fallait l'écart » et appliquait autre
   chose. *Un motif juste — « mesurer un delta » — cesse de l'être hors du cas qui l'a produit.*
3. ⛔ **On écrit l'avertissement d'un côté et on commet la faute de l'autre.** Le commentaire du
   410 attribuait un silence au `rollback` soixante lignes au-dessus d'un test qui interdit
   précisément cela. *Écrire une règle ne protège pas de l'enfreindre dans le même fichier.*
4. ⛔ **Le défaut de production est un manqué de propagation** : le principe — *« une clé absente
   vaut `null` et EFFACE ; un effacement par omission ne laissait aucune trace »* — était écrit
   pour `companies` et n'avait pas été porté à `users`, dont la route a la **même** sémantique.

**Gate après remédiation** : base reconstruite par les **trois** étapes et vérifiée, `fmt` propre,
`clippy --workspace --all-targets -D warnings` propre, **2320/2320** (4 skipped, 90 s) —
2318 + 2 tests neufs.

⚠️ **La boucle NE PEUT PAS se clore sur la passe 3** : P2-5 s'est corrigé **dans du code de
production** (`routes/users.rs`), et la § *La passe ciblée* exige alors une passe de plus braquée
sur ce patch.

