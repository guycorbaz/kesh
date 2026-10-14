# Epic 29 — registre des choix autonomes, Story 29-4 (à verser par union dans `epic-29-choix-autonomes.md`)

*Fiche : `29-4-cloture-compte-concurrente.md`. Issue : #651 (P2). Choix faits le 2026-10-14, à la création de la fiche
(Opus 5.5, autonomie).*

---

## Story 29-4 — Une saisie concurrente ne passe plus sur un compte qu'on clôt

### C-29-4-1 — Fermer la fenêtre par une revérification verrouillante partagée, complétée du rejeu

- **Contexte** : L1 de la 28-4 (#651). La garde non verrouillante lit la vue de l'écrivain ; la clôture ne prend que
  sa propre ligne en exclusif. Sans conflit entre les deux, rien ne les sérialise. La 28-4 a écarté une garde
  verrouillante (exclusive, posée tôt) pour ses cycles (P1 F1/F4, P2 R1/F1/F2/F5), puis a déplacé la vue en la rendant
  non verrouillante à la même place (P3 R1).
- **Retenu** : piste (c) — une lecture **verrouillante partagée** de `closed_on` (`LOCK IN SHARE MODE`, clé primaire),
  `bank_accounts::recheck_open_locked_in_tx`, sur les sept chemins à fenêtre réelle ; complétée de (b), le rejeu de la
  clôture et de la réouverture (C-29-4-5). Les gardes non verrouillantes de la 28-4 restent (C-29-4-3).
- **Écartées** :
  - (a) verrou commun dans un ordre global unique : le dépôt n'en a pas (`journal_entries.rs:1358`) ; l'ordre
    « document → compte → exercice » ferme un cycle avec `POST`, `PUT`, `DELETE /bank-accounts`, non rejouées ;
  - (b) seule : pas de verrou commun, donc pas de cycle à rejouer — la fenêtre n'est pas un interblocage ;
  - (d1) verrou nommé (`GET_LOCK`) pris aussi par la clôture : `with_account_lock` le relâche avant le commit de
    l'appelant (`mutex.rs:131-136`), et un cycle verrou nommé / verrou de ligne n'est pas détecté (`1205`) ;
  - (d2) la clôture verrouille les lignes enfants : ensemble de verrous large, analyse que la 28-4 a dû abandonner ;
  - (d3) refuser la clôture pendant une écriture, ou documenter la fenêtre : inutile, (c) la ferme ;
  - verrou **exclusif** de l'écrivain : rouvre les cycles écrivain / écrivain de la 28-4 (W9 / W24, W30 / W24).
- **Réversible** : oui — retirer sept appels et deux enveloppes rétablit la 28-4.
- *Annotation du 2026-10-14 (remédiation de la validation P1)* : le complément (b) ne porte plus que sur la **clôture**
  (C-29-4-5 amendé) ; s'y ajoute l'épinglage d'`innodb_snapshot_isolation` (C-29-4-13). Réversible : sept appels, une
  enveloppe, un réglage de session.
- *Annotation du 2026-10-14 (remédiation de la validation P2 ; décisions de l'orchestrateur)* : le complément (b) porte
  désormais sur la **clôture, la réouverture et le `PATCH` du compte** (C-29-4-5, C-29-4-14 renversé, C-29-4-19) ;
  l'épinglage d'`innodb_snapshot_isolation` est **retiré** (C-29-4-13, #669) ; la revérification a deux variantes
  (C-29-4-20). Réversible : sept appels, deux fonctions, trois enveloppes.

### C-29-4-2 — Positions : après l'`INSERT` à clé étrangère (W17, W22) ; en dernière acquisition de verrou (annulations)

- **Contexte** : la position d'un verrou neuf décide de ses cycles ; celle d'une lecture non verrouillante, de la vue.
- **Retenu** : W22 juste après `INSERT INTO bank_imports`, W17 juste après `INSERT INTO payment_batches` — le verrou
  partagé du compte y est **déjà tenu** (clé étrangère ; T-a31 le vérifie), la demande ne crée rien ; W9, W24, W25,
  W28, W29 **après leur dernière écriture**, avant l'audit — le compte vient après `companies` (ordre des routes de
  gestion, aucun cycle avec elles) et après l'exercice (cycle avec W8 / W27, rejoués des deux côtés).
- **Écartées** : juste après le verrou du document (cycle avec les routes de gestion non rejouées — R1 de la P2 de la
  28-4) ; au point de la garde de la 28-4 (entre l'exercice et `companies` : même cycle) ; W22 avant l'`INSERT` (verrou
  explicite à une position non analysée, sans gain).
- **Réversible** : oui (une ligne par site).

### C-29-4-3 — Garder les gardes non verrouillantes de la 28-4

- **Contexte** : la revérification seule suffirait à refuser.
- **Retenu** : les garder — refus précoce sans écrire, et ordre des refus fixé par la 28-4 (W24 `HasReconciled` avant
  « clos », motifs d'exercice après la garde, T-a27) ; la revérification ne répond que dans la course.
- **Écartée** : les remplacer par la revérification (l'ordre des refus changerait : un compte clos dans un exercice
  clos rendrait le motif d'exercice au lieu de `BANK_ACCOUNT_CLOSED`).
- **Réversible** : oui.

### C-29-4-4 — Prémisses de #651 corrigées : W30 sans fenêtre, W24 seulement pour un import vide

- **Contexte** : #651 liste huit chemins, dont l'annulation de rejet et la suppression d'import.
- **Retenu** : W30 n'a **pas** de fenêtre (I1 : une rejetée est `pending`, la clôture refuse tant qu'elle existe ; le
  test de la 28-4 ne monte la course qu'en posant `closed_on` par SQL direct) ; W24 n'en a une que pour un **import
  vide** (sinon ses transactions `pending` font refuser la clôture). W24 reçoit quand même la revérification ; W30 non.
  À commenter sur l'issue par l'orchestrateur.
- **Réversible** : sans objet (constat).

### C-29-4-5 — Rejouer la clôture et la réouverture

- **Contexte** : en attente de sa ligne, la clôture ne tient rien, mais sa demande exclusive fait attendre derrière elle
  les demandes partagées ultérieures : elle peut fermer un cycle à trois, dont elle est la victime (la plus légère).
- **Retenu** : `retry_app_on_deadlock("bank_accounts::close" / "::reopen")` autour d'une fonction « une tentative » ;
  point (vi) d'`audit_route_registry.rs` passe de sept à neuf routes ; un test à trois connexions (T-a4).
- **Écartée** : ne rien faire (un `500` rare sur la clôture) ; rejouer aussi les routes de gestion (aucun cycle neuf ne
  les atteint, C-29-4-2).
- **Réversible** : oui.
- *Amendé le 2026-10-14 (validation P1, F5)* : **la réouverture n'est pas rejouée**. Pour qu'elle soit victime d'un cycle
  à trois, il faudrait un détenteur partagé de la ligne d'un compte **clos** en attente d'une ressource ; toute écriture
  sur un clos est refusée avant de prendre ce partagé (garde non verrouillante ou lecture sous `FOR UPDATE`), et les
  gestes permis sur un clos (W21, W30) ne touchent pas la ligne du compte. Point (vi) : sept → **huit** (non neuf) ; le
  test T29a-4 ne porte que sur la clôture. Écartée : garder les deux comme défense en profondeur (élargit le périmètre
  minimal sans cas à défendre).
- *Amendé le 2026-10-14 (validation P2, F3 = R9 ; décision de l'orchestrateur)* : l'amendement de la P1 est **renversé**
  — la réouverture est rejouée (C-29-4-14). Point (vi) : sept → **dix** (clôture, réouverture, `PATCH`).

### C-29-4-6 — Découper en 29-4a (import, lot, rejeu) et 29-4b (annulations, textes)

- **Contexte** : plus de cinq modules au total ; le risque n'est pas réparti également.
- **Retenu** : 29-4a sans verrou neuf d'écrivain (quatre modules) ; 29-4b porte le verrou neuf, son cycle et les textes
  (cinq modules, la borne). Une fiche, deux clés (précédent C-28-4-1) ; `refs #651` puis `closes #651` ; aucune release
  entre les deux (la v0.13.4 attend la 29-4b).
- **Écartées** : une seule story (dérogation, comme la 28-4a — non nécessaire ici, la moitié a est cohérente seule :
  l'inventaire de l'AC5 dit `Ouverte("29-4b")` pour les cinq sites restants) ; trois stories (le rejeu seul n'a pas
  d'objet sans la 29-4b ; il est rangé avec la fonction de revérification, même module).
- **Réversible** : oui.
- *Annotation du 2026-10-14 (validation P1, R6 = F7)* : « cinq modules, la borne » était un **sous-décompte** — la 29-4b
  touche aussi des doc-comments de `bank_accounts.rs` et de `routes/reconciliation.rs`. Recompté (fiche, § *Périmètre
  minimal*) : 29-4a cinq modules de code (`pool` compris, C-29-4-13), 29-4b quatre de code et sept comptée largement ;
  le découpage reste, la 29-4b porte une dérogation motivée (C-29-4-18).
- *Annotation du 2026-10-14 (validation P2, R1 = F1)* : `pool` sort du décompte avec l'AC11 ; la 29-4a touche **quatre**
  modules de code (les deux enveloppes neuves et la variante « archivé » vont dans des modules déjà comptés), sept
  distincts au total ; le découpage reste, la dérogation de la 29-4b reste (relue, inchangée), aucune n'est due pour la
  29-4a. Le signal D5 levé en P2 (HIGH né de la remédiation P1) est déclaré au Project Lead (Change Log).
- *Annotation du 2026-10-14 (validation P3, R4 = F4)* : le « quatre » de l'annotation précédente était faux **à sa
  date**, d'un module : la fixture `attendre_un_verrou_en_attente` que la même remédiation P2 ajoutait à
  `kesh-db/src/test_fixtures.rs` (compilé en permanence, hors `cfg(test)`) en faisait cinq. La P3 retire la fixture
  (C-29-4-21) : la 29-4a touche de nouveau **quatre** modules de code — et, comptée largement, six fichiers de
  production, deux doc-comments seuls s'ajoutant (R8 de la P3 ; C-29-4-18). Le découpage reste.

### C-29-4-7 — Résidu : un cycle à trois où l'import serait victime

- **Contexte** : import (partagé depuis son `INSERT`, en attente de l'intervalle d'une suppression d'import), suppression
  d'import (revérification en attente derrière la clôture), clôture en attente : cycle à trois sur un même compte.
- **Retenu** : assumé, écrit au doc-comment de la revérification ; la clôture, plus légère, est la victime désignée
  (prémisse tenue par le dépôt, `rejeu_interblocage_e2e.rs:9-11`), et elle est rejouée. Classe LOW : aucune issue.
- **Écartée** : rejouer l'import (extraction de ~160 lignes sur deux chemins, CAMT et CSV) — hors périmètre minimal.
- **Réversible** : oui (une enveloppe à ajouter si le cas se produit).
- *Amendé le 2026-10-14 (validation P1, F8 b)* : la réserve « et que la transaction la plus légère ne soit pas
  désignée » décrivait un cas qui n'arrive qu'à égalité de poids ; dans ce cycle, la clôture (une demande en attente,
  rien d'écrit) est toujours la plus légère. Le résidu est **théorique**, écrit comme tel, sans « à surveiller ».
- *Annoté le 2026-10-14 (validation P2, F2)* : le même cycle existe avec le `PATCH` du compte à la place de la clôture
  comme demandeur d'exclusif en file ; le `PATCH` est désormais rejoué (C-29-4-19), et l'import, qui a écrit, reste le
  plus lourd : même résidu théorique. L'argument de poids est écrit (fiche, § *Cycles*, F11 de la P2).
- *Annoté le 2026-10-14 (validation P3, F3)* : « la clôture est toujours la plus légère » est **raisonné** : le dépôt
  n'établit le choix du plus léger que dans des cycles à deux, et ce cycle-ci en compte trois. Si InnoDB ne comparait
  que la paire qui ferme le cycle, l'import ou la suppression pourraient être désignés. T29a-4 et T29a-9, premiers
  tests à trois, portent une seconde contingence (victime observée consignée, remontée à l'orchestrateur) ; le résidu
  reste écrit théorique sous cette réserve.

### C-29-4-8 — Pas de revérification là où un invariant ou un verrou existant tient

- **Retenu** : W30, W1-W6 et `accept_one_*` (I1), W20 (`pay_in_tx` sous `FOR UPDATE`, et I2), W8, W27, W13-W15
  (`FOR UPDATE` de la ligne) : aucun appel ; le troisième inventaire du registre les classe `SansFenetre` avec leur
  raison.
  *Annoté le 2026-10-14 (revue de code P1 de la 29-4a, A-5)* : la variante s'appelle `Fermee` au code
  (`bank_account_closed_inventory.rs`), renommée au développement (C-29-4-30).
- **Réversible** : oui.

### C-29-4-9 — Le critère décidable : trois volets

- **Retenu** : (1) verrou neuf déjà tenu ou dernière acquisition ; (2) tout cycle fermé a tous ses participants rejoués
  (ou passe par la clôture, rejouée) ; (3) aucune naissance de vue déplacée (hérité de l'AC7 de la 28-4). Il remplace,
  pour les chemins touchés, le volet « aucun verrou ajouté » de la 28-4, que fermer la fenêtre ne permet pas de tenir.
- **Réversible** : sans objet.

### C-29-4-10 — Égalité sur la clé primaire, contrôlée par `EXPLAIN`

- **Contexte** : un verrou d'intervalle (plan par l'index de société) ajouterait des verrous et romprait « déjà tenu ».
- **Retenu** : `WHERE id = ? AND company_id = ?` (accès `const` sur `PRIMARY`), et un test `EXPLAIN`.
- **Réversible** : oui.

### C-29-4-11 — Textes : manuel et API externe en 29-4b ; CHANGELOG par union dans chaque PR ; `[0.13.3]` intouché

- **Retenu** : la 29-4a écrit les doc-comments et la ligne du CHANGELOG de ce qu'elle ferme ; la 29-4b réécrit L1 au
  manuel (PDF), à `api-external.md` et aux doc-comments restants. La section `[0.13.3]` du CHANGELOG, la fiche et le
  registre de la 28-4 sont des relevés datés : ils ne se réécrivent pas.
- **Réversible** : oui.

### C-29-4-12 — Un seul test du cycle neuf (W25), non cinq

- **Contexte** : les cinq annulations ferment le même cycle (compte X → exercice contre exercice → compte S), avec des
  partenaires tous rejoués au sol.
- **Retenu** : T-b3 sur W25 prouve que le cycle est détecté (`1213`) et rejoué ; pour les quatre autres, la table des
  cycles et les enveloppes existantes (`audit_route_registry.rs`).
- **Écartée** : cinq tests à trois connexions (coût, sans information neuve).
- **Réversible** : oui.
- *Annotation du 2026-10-14* : le test se nomme désormais **T29b-3** (C-29-4-17).

### C-29-4-13 — Épingler `innodb_snapshot_isolation = OFF` par session dans `create_pool`

- **Contexte** (validation P1, R4 = F6 ; décision de l'orchestrateur : la revérification ne doit pas dépendre en silence
  de ce réglage) : sous `ON` — défaut de MariaDB à partir de la 11.6 —, une lecture verrouillante qui rencontre une
  version postérieure à la vue rend `ER_CHECKREAD` (1020), dans la course même que la revérification traite. Établi au
  code : `map_db_error` ignore 1020, qui sort en `DbError::Sqlx` → `500 INTERNAL_ERROR`. Constaté : base de dev
  `10.11.16`, variable présente, `OFF`. Le dépôt dépend déjà de `OFF` ailleurs (`routes/reconciliation.rs:2166`,
  `bank_imports.rs:439`). Version du NAS inconnue.
- **Retenu** : `configurer_session` (appelée par `after_connect`) exécute `SET SESSION innodb_snapshot_isolation = OFF`,
  en ignorant la seule erreur 1193 (variable inconnue d'un serveur ancien, qui se comporte comme `OFF`) ; test T29a-8
  (constat de 1020 sous `ON`, puis épinglage) ; mutation M29-5.
- **Écartées** : (1) traduire 1020 en `BANK_ACCOUNT_CLOSED` sur le chemin de la revérification — faux quand la
  modification postérieure est un `PATCH` ou un renommage, et ne couvre pas l'`INSERT`, dont le contrôle de clé
  étrangère est lui aussi une lecture verrouillante (non établi) ; (2) traduire 1020 en rejeu — l'import et la création
  de lot ne sont pas rejoués (C-29-4-7) ; (3) vérification au démarrage avec refus de démarrer — bloquerait une
  installation à la montée de version du serveur pour un réglage que l'application peut poser elle-même ;
  (4) exigence de version au manuel d'administration seule — laisse le `500` à qui ne lit pas le manuel.
- **Réversible** : oui (une instruction).
- *Annotation du 2026-10-14 (validation P2, R1 = F1, HIGH convergent ; décision de l'orchestrateur)* : **retiré de la
  story**. `create_pool` n'a **aucun appelant** — la production construit son pool dans `kesh-api/src/main.rs:74`, sans
  `after_connect`, depuis la Story 1.2 — : le réglage n'aurait atteint aucune connexion, et T29a-8 (b), qui appelait
  `configurer_session` directement, aurait certifié une prémisse fausse. Retrait plutôt qu'ajout : brancher le réglage
  sur le pool réel touche toute la base (`sql_mode`, `time_zone` décrits par `create_pool` et jamais appliqués) et
  ajoute un module à la 29-4a. Sortent : l'AC11, `configurer_session`, T29a-8, M29-5. Question portée par **#669**
  (bug, P3). Angle mort écrit : sous MariaDB 11.6+, la revérification rendrait `1020` → `500` dans la course, comme les
  deux sites préexistants. Les écartées (1) à (4) ci-dessus restent écartées pour la story ; #669 les rouvre.

### C-29-4-14 — La réouverture n'est pas rejouée

- Voir l'amendement de C-29-4-5 (validation P1, F5). Réversible : oui (une enveloppe).
- *Renversé le 2026-10-14 (validation P2, F3 = R9 ; décision de l'orchestrateur)* : **la réouverture est rejouée**.
  L'argument de la P1 était faux : « toute écriture sur un clos est refusée **avant** de prendre ce partagé » — W22 et
  W17 prennent le partagé du compte par leur `INSERT` avant que la revérification les refuse, les annulations le
  prennent par la revérification ; et une demande partagée attend derrière une demande exclusive **en attente**, celle
  de la réouverture comprise. Cycle à quatre (raisonné) : annulation → réouverture → import → route de gestion →
  annulation. Retenu : `reopen_once` et `retry_app_on_deadlock("bank_accounts::reopen", …)`, T29a-9, M29-20.
  **Écartée** : rejouer aussi les routes de gestion (`PUT`, `POST`, `DELETE /bank-accounts`) pour fermer le résidu de
  ce cycle — `M` y est à égalité de poids avec la réouverture et peut être désignée, `500` sans rien d'écrit ; quatre
  gestes concurrents sur un même compte et une même société, aucun cas observé : trois enveloppes de plus pour un cas
  négligeable, résidu écrit. Réversible : oui (une enveloppe).
- *Annoté le 2026-10-14 (validation P3, F2 ; décision de l'orchestrateur de garder l'écart)* : le résidu était
  inventorié pour la seule réouverture. Il se forme avec **tout** demandeur d'exclusif en file sur le compte —
  clôture, réouverture, `PATCH`, W8, W27 — et `M`, qui ne tient aucun verrou d'enregistrement, est à égalité de poids
  avec la clôture et la réouverture, **strictement plus légère**, donc victime désignée, contre le `PATCH`, W8 et W27.
  La décision ne change pas (routes de gestion non rejouées, quatre gestes concurrents, `500` sans rien d'écrit) ;
  l'inventaire du résidu est réécrit dans la fiche (critère volet 2, AC4, § *Cycles*, angles morts) et au doc-comment
  prescrit.
- *Annoté le 2026-10-14 (revue de code P1 de la 29-4a, E-1 ; décision de l'orchestrateur)* : **l'écart est levé** —
  les routes de gestion (`PUT`, `POST`, archivage) sont rejouées (C-29-4-28). Le résidu se ferme : `M`, désignée,
  rejoue. Ce qui a fait changer la décision n'est pas le cycle à quatre de la 29-4b mais le cycle **préexistant**
  `I ↔ M` (et `I → M → E → I`), atteignable dès la 29-4a, où `M` était la victime en `500`.

### C-29-4-15 — L'ensemble des gardes non verrouillantes s'extrait du code, et `FENETRE` lui est égal

- **Contexte** (validation P1, R3) : l'assertion (ii) de l'AC5 sélectionnait les sites par le **texte** des raisons du
  registre principal — que la 29-4b réécrit —, manquait W20 et rangeait W1-W6 / `accept_one_*`, qui sont des
  `Invariant`, à `FENETRE`.
- **Retenu** : `G` = appelants d'`ensure_open_in_tx` / `ensure_open_unarchived_in_tx` ∪ fonctions portant un littéral
  qui lit `closed_on` `FROM bank_accounts` sans `FOR UPDATE` ni `LOCK IN SHARE MODE` (hors `bank_accounts.rs`) ;
  assertion d'**égalité** `G` = fonctions de `FENETRE` ; neuf entrées au sol (W9, W17, W20, W22, W24, W25, W28, W29,
  W30).
- **Écartées** : un champ `Lecture::{Verrouillante, NonVerrouillante}` ajouté au registre principal (27 entrées à
  retoucher, et le champ resterait une déclaration, non une extraction) ; la sélection par texte (change de sens à la
  réécriture d'une raison).
- **Réversible** : oui.

### C-29-4-16 — La course archivage / import et lot (L5) est nommée, non fermée

- **Contexte** (validation P1, F8 a) : la revérification, partagée avec les annulations, ne lit pas `archived` ; un
  archivage commité entre la garde de tête de W22 / W17 et leur `INSERT` laisse passer l'écriture sur un compte archivé.
  Le dépôt nomme déjà cette course pour l'import (« L5 de la route d'archivage », `bank_accounts.rs:866-867`), sans issue.
- **Retenu** : angle mort écrit (fiche, Dev Notes ; doc-comment de `recheck_open_locked_in_tx`) ; **issue à ouvrir** par
  l'orchestrateur (course archivage / import et archivage / lot, L5).
- **Écartée** : une variante `recheck_open_unarchived_locked_in_tx` pour W22 / W17 — hors #651, et non mécanique (W17
  rend `IllegalStateTransition` pour un archivé, W22 `NotFound` : deux traductions).
- **Réversible** : oui.
- *Renversé le 2026-10-14 (validation P2, R2 = F4 ; décision de l'orchestrateur)* : la course **est fermée**, et la
  variante écartée est retenue — elle est mécanique aux conditions que la P2 a écrites (C-29-4-20). Plus d'issue L5 à
  ouvrir ; l'angle mort disparaît de la fiche ; le doc-comment `bank_accounts.rs:866-867` se réécrit en 29-4a.

### C-29-4-17 — Identifiants propres à la story : `T29a-n`, `T29b-n`, `M29-n`

- **Contexte** (validation P1, R7) : `T-a1` à `T-a5` existent dans le module `compte_clos` (28-4), `T-b1` à `T-b4` dans
  `bank-accounts.api.test.ts`, `M1` à `M15` dans les doc-comments de la 28-4.
- **Retenu** : renumérotation de la fiche ; `T-a31`, `T-a45`, `T-a48`, `T-a49` gardent leur sens (tests de la 28-4).
- **Réversible** : oui.

### C-29-4-18 — Dérogation à la règle de splitting pour la 29-4b

- **Contexte** (validation P1, R6 = F7) : comptée largement, la 29-4b touche sept modules.
- **Retenu** : pas de redécoupage ; ce qui dépasse cinq n'est que du texte (deux fichiers à doc-comments seuls, le
  manuel), le code est un rollout mécanique du patron de la 29-4a ; séparer textes et code ferait vivre plus longtemps
  des textes faux. Section *Dérogation règle de splitting (29-4b)* de la fiche.
- **Écartées** : une 29-4c pour les textes ; déplacer en 29-4a la réécriture de `bank_accounts.rs` (fausse tant que les
  annulations restent ouvertes).
- **Réversible** : oui.
- *Annoté le 2026-10-14 (validation P3, R8)* : la 29-4a, comptée largement, touche elle aussi plus de cinq fichiers de
  production — quatre modules de code et deux fichiers de doc-comments seuls (`kesh-db/src/errors.rs`,
  `kesh-api/src/routes/bank_imports.rs`), soit six. Même argument, même conclusion : ce qui dépasse cinq n'est qu'une
  phrase de doc-comment que la revérification rend incomplète ; la fiche étend la section *Dérogation* à la 29-4a.
- *Annoté le 2026-10-14 (validation P4, L4)* : le « sept » de la 29-4b comptée largement ne suivait aucune règle. Règle
  énoncée : fichiers de production (code ou doc-comments), sans documents ni tests — 29-4a **six**, 29-4b **six** (quatre
  de code, deux de doc-comments seuls), **neuf** avec ses trois documents. Conclusion inchangée.

### C-29-4-19 — Rejouer le `PATCH` du compte (décision de l'orchestrateur, validation P2)

- **Contexte** (validation P2, F2) : la revérification de la 29-4b crée un cycle à trois par la file d'attente — un import
  `I` qui a passé sa revérification tient le compte en partagé et attend l'intervalle d'`idx_bank_transactions_import`
  tenu par une suppression d'import `D` (W24) ; un `PATCH` du compte demande l'exclusif derrière `I` ; `D`, à sa
  revérification, attend derrière le `PATCH` en file. Le `PATCH`, qui n'a rien écrit, est le plus léger : victime,
  `500` sans enveloppe. C'est aussi lui la victime de l'angle mort « PATCH » par `accounts` écrit en P1.
- **Retenu** : `patch_once` (la transaction du `PATCH` : partagés des réglages, `FOR UPDATE` du compte, audit, commit)
  enveloppée par `retry_app_on_deadlock("bank_accounts::patch", …)` ; version et `validate_journal_account_id` devant
  l'enveloppe ; `rollback` explicite sur refus ; test T29a-10 (patron `victime()` : le `PATCH` tient les partagés des
  réglages que la transaction de test demande) ; mutation M29-21. Ferme le cycle et l'angle mort d'un seul geste.
- **Écartées** : écrire le cycle comme troisième angle mort (F2, option b : deux angles morts au lieu d'une enveloppe) ;
  monter le cycle à trois dans le test (le cycle à deux suffit à prouver que la victime est rejouée).
- **Réversible** : oui (une enveloppe).

### C-29-4-20 — Deux variantes de la revérification ; W17 convertit l'archivé (décision de l'orchestrateur, validation P2)

- **Contexte** (validation P2, R2 = F4) : la revérification de l'import et du lot doit lire `archived` (fermer L5), celle
  des annulations jamais (un archivé ne se rouvre pas : refuser l'annulation d'un de ses règlements la rendrait
  définitivement impossible).
- **Retenu** : `RECHECK_OPEN_LOCKED_SQL` / `recheck_open_locked_in_tx` (annulations) et
  `RECHECK_OPEN_UNARCHIVED_LOCKED_SQL` / `recheck_open_unarchived_locked_in_tx` (W22, W17), symétrie exacte
  d'`ENSURE_OPEN_SQL` / `ENSURE_OPEN_UNARCHIVED_SQL` ; ordre « archivé avant clos » ; W22 rend `NotFound` (traduit en
  `404` par la route, rien à ajouter) ; W17 convertit `NotFound` en `IllegalStateTransition("le compte bancaire source est
  archivé")` au site d'appel (sûr : la ligne existe après l'`INSERT` à clé étrangère, sa société est contrôlée à
  l'étape (0)) ; registre `Reverifiee(Variante, …)` et assertion (i bis) en dur pour les annulations.
- **Écartées** : une seule fonction qui rendrait `archived` à l'appelant (chaque appelant redécide, et une annulation
  pourrait le lire) ; faire lire `archived` à la fonction partagée (régression : annulations impossibles sur un
  archivé) ; traduire l'archivé de W17 en `NotFound` (changerait le refus de l'étape (0)).
- **Réversible** : oui.

### C-29-4-21 — Établir l'attente par `INNODB_TRX`, non par un délai (T29a-6, T29a-7)

- **Contexte** (validation P2, R4 = F7) : dans l'autre ordre, l'issue ne départage pas « prémisse vraie » de « montage
  arrivé avant que la demande exclusive soit en file » ; un délai de 500 ms l'espérait, et la sonde de requête en cours
  ne voit pas une requête bloquée (P5 de la 28-4).
- **Retenu** : la tâche de la clôture relève son `CONNECTION_ID()` ; le test attend une ligne d'`INNODB_TRX` avec
  `trx_mysql_thread_id` égal et `trx_state = 'LOCK WAIT'`, bornée à 10 s (fixture neuve `attendre_un_verrou_en_attente`).
  Colonnes présentes sous MariaDB 10.11 : raisonné, à vérifier au dev.
- **Écartées** : garder le délai seul (passe à vide sans signal) ; `INNODB_LOCK_WAITS` (même information, une jointure de
  plus) ; écrire seulement au Dev Agent Record que le test ne distingue pas les deux cas.
- **Réversible** : oui.
- *Annoté le 2026-10-14 (validation P3, R1 = F1 ; décision de l'orchestrateur)* : **la sonde `INNODB_TRX` est
  remplacée**. Elle exige le privilège `PROCESS`, que les droits versionnés de l'utilisateur de dev ne donnent pas
  (`scripts/mariadb-init/01-dev-grants.sql`, deux espaces de noms) — le dépôt le disait deux fois
  (`reconciliation_e2e.rs:7546-7553`, `:8394-8397`) ; « colonnes présentes : raisonné » visait le mauvais obstacle.
  **Retenu** : le helper existant `sonde_verrou_nowait` (`test_fixtures.rs:607`), en boucle locale au test, avec
  `SELECT id FROM bank_accounts WHERE id = ? LOCK IN SHARE MODE NOWAIT` sur une connexion tierce : compatible avec le
  seul partagé de l'écrivain, il échoue en `1205` dès que la demande exclusive de la clôture est en file. Aucun
  privilège, aucune fixture neuve — la fixture `attendre_un_verrou_en_attente` est retirée, et la 29-4a revient à quatre
  modules de code (C-29-4-6). **Établi** le 2026-10-14 sur la MariaDB de dev (`10.11.16`, base `kesh_294`, table
  jetable, trois connexions) : succès avec un partagé tenu, `1205` aussitôt avec un exclusif en file, succès après les
  commits ; à rejouer dans le test. **Repli**, si la branche rebasée le contredisait : le précédent
  `reconciliation_e2e.rs:7546-7553` (attente bornée constatée, issue après le commit) et le constat écrit au Dev Agent
  Record que le test ne distingue plus les deux cas. **Écartées** : accorder `PROCESS` à l'utilisateur de dev (droit
  global, contraire à la justification de `01-dev-grants.sql`, hors des modules de la story) ; garder `INNODB_TRX`
  parce que le conteneur de la station porte, ce jour, un `GRANT ALL PRIVILEGES ON *.*` à `kesh` étranger au fichier
  versionné (le test passerait ici et rougirait sur un conteneur rejoué).
- *Annoté le 2026-10-14 (validation P4, M1, L1, L5)* : la garde contre le montage à vide est une **sonde de référence
  entre les étapes (1) et (2)**, avant le lancement de la clôture — placée après, la clôture pouvait déjà être en file et
  la garde donner un faux rouge ; la forme `LOCK IN SHARE MODE NOWAIT` n'a aucun précédent dans le dépôt (constat hors dépôt
  seulement), elle s'établit au T0 du développement, repli inchangé ; le motif « 500 ms » du repli se lit à
  `reconciliation_e2e.rs:7538-7553`.

### C-29-4-22 — Un seul test « annulation sur un compte archivé » (W25), non cinq

- **Contexte** : les cinq annulations appellent la même fonction `recheck_open_locked_in_tx`.
- **Retenu** : T29b-5 sur W25 (règlement par virement, compte archivé ensuite — l'archivage ne compte que les
  `bank_transactions`) ; pour les quatre autres, l'assertion (i bis) du registre, qui interdit en dur la variante aux
  annulations ; mutation M29-24.
- **Écartée** : cinq tests fonctionnels (coût, sans information neuve).
- **Réversible** : oui.

---

*Choix du développement de la 29-4a (`bmad-dev-story`, Opus 5.5, autonomie, 2026-10-14).*

### C-29-4-23 — Répartition des tests de la partie a

- **Contexte** : l'AC3 place les tests de fenêtre au module `compte_clos` de `bank_accounts_repository.rs` ; ceux de
  W17 exigent une facture fournisseur payable et un compte source lié au grand livre, montage que porte déjà
  `payment_batches_repository.rs` (`setup`, `make_invoice`).
- **Retenu** : T29a-1, T29a-2, T29a-6, T29a-7, T29a-11 au module `compte_clos` ; T29a-3 et T29a-12 dans
  `payment_batches_repository.rs`, qui réutilisent son montage (le compte source y est rendu non principal par SQL,
  un principal ne se clôt pas). T29a-7 reste au module `compte_clos` : il insère son lot par SQL et n'a besoin
  d'aucune facture.
- **Écartée** : recopier le montage fournisseur dans `bank_accounts_repository.rs` (duplication).
- **Réversible** : oui (déplacement de tests).

### C-29-4-24 — La position d'une revérification, déclarée en structure et non en texte

- **Contexte** : l'AC5 écrit `Reverifiee(Variante, "<position>")` et exige un ordre textuel contrôlé (assertion
  (iii)).
- **Retenu** : `Reverifiee(Variante, Position { apres, avant })` — `apres` et `avant` sont des fragments SQL cherchés
  dans les littéraux de la fonction (aplatis : échappements neutralisés, blancs réduits, majuscules) ; l'appel doit
  suivre le dernier littéral qui contient `apres` et précéder le premier qui contient `avant`. La position devient
  l'entrée même de l'assertion, au lieu d'un texte que rien ne relirait. La 29-4b pourra élargir `Position` aux
  jetons de code et à la « dernière occurrence de tout geste verrouillant » de l'AC8.
- **Écartée** : une chaîne libre doublée d'une table d'ordre séparée (deux sources pour un fait).
- **Réversible** : oui.

### C-29-4-25 — Une conclusion commune aux trois tentatives ; `begin` et `commit` gardent leur rendu

- **Contexte** : `close_once`, `reopen_once` et `patch_once` concluent de la même façon (commit, ou `rollback`
  explicite avant de rendre l'erreur). Le patron `delete_once` rend les erreurs de `begin` / `commit` en
  `AppError::Database`, les trois routes en `AppError::Internal`.
- **Retenu** : une fonction `conclure(tx, result)` partagée (DRY) ; le rendu `Internal` de `begin` et `commit`
  **inchangé** — un `commit` InnoDB ne rend pas d'interblocage (les cycles se détectent à la demande de verrou),
  et changer le code d'erreur d'une panne de connexion est hors périmètre.
- **Écartée** : recopier le `match` final trois fois ; aligner sur `delete_once`.
- **Réversible** : oui.

### C-29-4-26 — T29a-4 et T29a-9 : attendre la demande de `t2` en cours avant que `t1` demande `R`

- **Contexte** : la séquence de l'AC4 enchaîne l'étape (3) (`t2` demande le compte en partagé, en tâche) et
  l'étape (4) (`t1` demande `R`) sans point de rendez-vous.
- **Retenu** : entre les deux, `attendre_une_requete_en_cours` sur la requête de `t2` — une aide d'ordonnancement, non
  une preuve (la sonde voit une requête en cours, non bloquée) ; c'est l'événement de rejeu de l'étape (5) qui
  établit l'attente, comme la fiche le prescrit. Le cycle se forme dans les deux ordres ; le rendez-vous évite
  seulement de dépendre de l'ordonnanceur pour que la demande de `t2` parte la première.
- **Écartée** : un délai fixe.
- **Réversible** : oui.

### C-29-4-27 — Dans les tests de fenêtre de l'import, la tâche commite sur succès

- **Contexte** : l'AC3 prescrit, pour W22, un `rollback` explicite de la tâche avant les comptages.
- **Retenu** : la tâche commite si l'import réussit — ce que fait la route — et annule explicitement sinon. Au sol,
  le test rougit sur l'issue (étape (5), `Ok(())` au lieu du refus) ; sous la story, l'annulation explicite précède
  les comptages, comme prescrit. Un `rollback` inconditionnel aurait rendu les comptages « rien d'écrit » vrais au
  sol aussi, sans rien prouver.
- **Écartée** : `rollback` inconditionnel.
- **Réversible** : oui.

### C-29-4-28 — Rejouer les routes de gestion du compte (décision de l'orchestrateur, revue de code P1, E-1)

- **Contexte** : la lentille E a reconstruit au code un cycle **préexistant** qu'aucun texte n'écrivait : un import ou
  un lot `I` tient la ligne du compte en partagé par le premier contrôle de clé étrangère de son `INSERT` et demande
  `companies` en partagé (le second) ; une route de gestion `M` (`PUT`, archivage, `POST` qui démote le principal)
  tient la sentinelle `companies` en exclusif et demande la ligne du compte en exclusif. À deux, `I ↔ M` ; à trois
  par la file, `I → M → E → I` quand la clôture, la réouverture ou le `PATCH` attend la ligne entre eux. `M`, non
  rejouée, plus légère que `I` (qui a écrit), en était la victime : `500`, rien d'écrit.
- **Retenu** : `create_once`, `update_once`, `archive_once` sous `retry_app_on_deadlock` (opérations
  `bank_accounts::create`, `::update`, `::archive`), sur le patron de la réouverture et du `PATCH`, conclusion par
  `conclure` ; contrôles de forme, onboarding, `validate_journal_account_id` et le pré-vol de l'archivage devant
  l'enveloppe. Tests T29a-13 (`PUT`), T29a-14 (archivage), T29a-15 (création), patron `victime()` et capture du rejeu ;
  mutations M29-26 à M29-28. Point (vi) du registre de rejeu : dix → treize. Résidu de C-29-4-14 fermé.
- **Écartée** : écrire le cycle comme résidu préexistant sans rejeu (remède de la lentille E).
- **Réversible** : oui (trois enveloppes).

### C-29-4-29 — La constante de lecture de `closing_checks.rs` entre au troisième inventaire (rebase sur la 29-1a)

- **Contexte** : au rebase sur `b4abae5d`, le troisième inventaire (T29a-5) a vu un site neuf : la constante
  `EXAMINED_ACCOUNTS_SQL` de `kesh-db/src/repositories/closing_checks.rs` (Story 29-1a) lit `closed_on` de
  `bank_accounts` sans verrou, hors de toute fonction. Ce n'est pas une garde : la préparation de la clôture d'un
  exercice liste les comptes à examiner, en lecture seule, et n'écrit rien.
- **Retenu** : une entrée `Fenetre::Fermee` pour `closing_checks.rs::<hors fonction>`, raison écrite ; décompte
  anti-muet de `G` : neuf → dix.
- **Écartée** : exclure le fichier de l'extracteur — une exclusion par fichier rendrait muet tout site futur du
  même fichier.
- **Réversible** : oui.

### C-29-4-30 — Écarts du développement à la fiche, versés au registre (revue de code P1, A-5)

- **Retenu, constaté au développement** : la variante `SansFenetre` de la fiche s'appelle `Fermee` au code (le nom
  dit le statut, non l'absence d'un site) ; trois mutations ajoutées à la campagne de la fiche — M29-3 bis (`LOCK IN
  SHARE MODE` retiré de `RECHECK_OPEN_UNARCHIVED_LOCKED_SQL`, la variante que W22 et W17 appellent : M29-3 ne visait
  que celle des annulations, sans appelant en 29-4a), M29-19 bis (`RECHECK_OPEN_LOCKED_SQL` en `FOR UPDATE`),
  M29-23 bis (W17 appelle `recheck_open_locked_in_tx`, la variante qui ne lit pas `archived`). Le Dev Agent Record les donnait « chacun au registre » sans qu'ils y soient.
- **Réversible** : oui.

### C-29-4-31 — Points reportés à la 29-4b par la passe ciblée P2 (annotation du 2026-10-14)

- **Retenu** : la passe ciblée P2 (Sonnet, 0/0/0/5) clôt la revue de la 29-4a ; R-3 est corrigé (CHANGELOG). Sont
  reportés à la 29-4b, sans toucher au code maintenant (fiche, T6-bis) : R-1 (T29a-15 joue un cycle synthétique, à dire
  dans le test et au point (vi)), R-2 (`was_primary` lu au pré-vol et réutilisé à chaque tentative d'`archive_once` : à
  relire dans la transaction de chaque tentative, avec un test — défaut préexistant que le rejeu élargit), R-4
  (doc-comments de cycles : distinguer mesuré et supposé), R-5 (mutation du `rollback` de `conclure`), et la liste des
  routes rejouées de `docs/api-external.md`.
- **Écartée** : corriger R-2 dans la 29-4a — chaque correctif de production rouvrirait la boucle de revue.
- **Réversible** : oui.

*Choix du développement de la 29-4b (Opus 5.5, autonomie, 2026-10-14).*

### C-29-4-32 — Les tests de fenêtre des annulations : dans `bank_accounts_closed_e2e.rs`, l'annulation appelée au dépôt

- **Contexte** : AC7 prescrit les tests T29b-1 « au dépôt, patron de l'AC3 », répartition au choix du dev. Les états à
  annuler (rapprochement `Entry`, règlement par virement, facture fournisseur payée par virement) se construisent par
  plusieurs routes ; `bank_accounts_closed_e2e.rs` en porte déjà les montages (`Ctx::rapprocher`, `regler`,
  `payer_fournisseur`).
- **Retenu** : T29b-1 (cinq tests), T29b-2 (deux) et T29b-5 dans `crates/kesh-api/tests/bank_accounts_closed_e2e.rs`.
  L'état se pose par les routes, sur un compte ouvert ; l'**annulation s'appelle au dépôt** (la fonction `_in_tx` que la
  route appelle, énumération `Annulation`), dans une transaction que le test possède — il voit le `DbError` et annule
  explicitement avant tout comptage ; la clôture concurrente est `close_in_tx`, au dépôt. T29b-5 passe, lui, par la
  route (l'annulation sur un compte archivé est un comportement de route). T29b-3 dans `rejeu_interblocage_e2e.rs`.
- **Écartée** : reconstruire ces états au dépôt dans `kesh-db/tests` (trois fichiers, montages de factures, d'écritures
  et de règlements à dupliquer) ; passer l'annulation par la route dans T29b-1 (l'erreur de dépôt, et le rollback
  explicite que l'AC3 exige, ne se voient plus).
- **Réversible** : oui (déplacement de tests).

### C-29-4-33 — R-2 : `is_primary` relu sous un `FOR UPDATE` avancé, par une fonction de dépôt neuve

- **Contexte** : T6-bis, R-2 de la passe ciblée P2 de la 29-4a — la garde « principal + autres comptes actifs » de
  l'archivage décidait sur le `is_primary` du pré-vol, réutilisé à chaque tentative du rejeu.
- **Retenu** : `bank_accounts::primary_flag_locked_in_tx` (`SELECT is_primary … AND archived = FALSE FOR UPDATE`),
  appelée dans `archive_once` juste avant la garde ; `None` → `404 BANK_ACCOUNT_NOT_FOUND`, le refus même
  qu'`archive_for_company` aurait rendu sur la même ligne. L'exclusif de la ligne, qu'`archive_for_company` prend
  ensuite, est **avancé** devant la garde : l'ordre « `companies` X puis compte X » est inchangé, et entre les deux
  positions la tentative ne fait qu'une lecture non verrouillante (raisonné). L'ordre des refus est inchangé
  (412 transactions → 412 principal → 409 version). Test `archive_rereads_the_primary_flag_inside_its_transaction`
  (`bank_accounts_e2e.rs`), rouge au sol : `412` au lieu de `409`.
- **Écartées** : une lecture non verrouillante après la sentinelle (juste tant que toute promotion passe par la
  sentinelle — l'onboarding `upsert_primary_in_tx` ne la prend pas) ; décider sur le `before.is_primary`
  d'`archive_for_company` (la garde passerait après le contrôle de version : ordre des refus changé).
- **Constat** : l'effet au sol est un **mauvais code de refus** (`412` sur un compte qui n'est plus principal), non un
  archivage indu — le verrou optimiste d'`archive_for_company` refusait déjà un compte dont la version avait bougé.
- **Réversible** : oui.
- *(Annotation du 2026-10-14, revue de code P1 de la 29-4b, A-5 et B-2.)* **Écart à la prescription** : T6-bis
  demandait « une promotion concurrente du compte **entre deux tentatives** » ; le cas joué est une **démotion**
  commitée entre le **pré-vol** et la **première** tentative (l'archivage attend la sentinelle). Le correctif est le
  même (relecture dans chaque tentative) ; le cas « entre deux tentatives » n'est pas joué. Et le test prouve la
  relecture **dans la transaction**, non le `FOR UPDATE` : un `SELECT` simple au même endroit y rendrait le même `409`
  (les bascules des routes de gestion sont déjà sérialisées par la sentinelle) ; le `FOR UPDATE` couvre l'intégration
  (`upsert_primary_in_tx`, sans sentinelle), non jouée. Écrit au doc-comment de `primary_flag_locked_in_tx`.
- *(Annotation du 2026-10-14, passe ciblée P2 de la revue de la 29-4b, R-1.)* **Prémisse fausse, retirée** : l'onboarding
  prend la sentinelle — `lock_company` (`FOR UPDATE` sur `companies`, `routes/onboarding.rs:585`) avant
  `upsert_primary_in_tx` (`:586`). Aucune promotion de production n'échappe à la sentinelle ; le motif « écarté » de la
  lecture non verrouillante (l. 482-483) et la « couverture de l'intégration » ci-dessus ne tiennent donc pas. Le
  `FOR UPDATE` reste, sans effet de justesse propre (il avance le verrou de la ligne qu'`archive_for_company` prend de
  toute façon) ; le doc-comment de `primary_flag_locked_in_tx` est corrigé.

### C-29-4-34 — R-5 : le `rollback` explicite de `conclure` gardé par un test textuel, son effet dynamique déclaré non couvert

- **Contexte** : R-5 demandait une mutation de `conclure` sans `rollback`, tuée par un test qui sonde le verrou, ou
  déclarée non couverte.
- **Établi au code** (sqlx 0.8.6) : `Transaction::drop` ne fait que mettre le `ROLLBACK` en file
  (`sqlx-mysql`, `start_rollback`), et `PoolConnection::drop` rend la connexion par une tâche lancée à part
  (`pool/connection.rs:208`) qui l'envoie (`ping`, `:314`). Sans `rollback` explicite, le verrou est rendu **un peu plus
  tard** ; un test qui le sonderait après la réponse ferait la course contre cette tâche.
- **Retenu** : T29b-7 (`bank_account_closed_inventory.rs`), test textuel sur les corps aplatis — `conclure` annule sur
  erreur, et les six tentatives s'y concluent ; il tue M29-35. Le comportement « verrou rendu avant la réponse » est
  déclaré non couvert dynamiquement (doc-comment du test, Dev Agent Record).
- **Écartée** : une sonde après la réponse (verte ou rouge selon l'ordonnanceur).
- **Réversible** : oui.

### C-29-4-35 — La position « en dernier » des annulations : une variante de `Position`, et deux jetons de plus

- **Contexte** : AC8 étend l'ordre textuel aux annulations (dernier geste verrouillant, avant l'audit) ; le registre de
  la 29-4a ne connaissait que `Position { apres, avant }` (littéraux).
- **Retenu** : `Position` devient une énumération — `Litteraux { apres, avant }` (import, lot) et `EnDernier` (les cinq
  annulations), contrôlée par `fautes_en_dernier` : jetons SQL de la fiche dans les littéraux, appels dans la copie
  masquée ; un corps sans aucun geste verrouillant est une faute (anti-muet). Aux appels de la fiche
  (`reverse_in_tx(`, `reverse_owned_in_tx(`, `cancel_settlement_in_tx(`) s'ajoutent les deux dissolutions du lettrage de
  pièce (`dissolve_invoice_document_group_in_tx(`, `dissolve_supplier_invoice_document_group_in_tx(`), qui écrivent :
  sans effet sur l'issue (elles précèdent la contre-passation), un angle mort de moins. Volet inverse :
  `the_last_acquisition_rule_reports_each_fault_on_built_cases` (sept cas).
- **Écartée** : une variante `Fenetre` de plus (la fiche écrit `Reverifiee(Variante::Clos, …)`).
- **Réversible** : oui.

### C-29-4-36 — Le paragraphe « Deux limites » du manuel devient « Deux précisions »

- **Contexte** : AC9 remplace la seconde puce (la fenêtre de #651) ; l'intitulé « Deux limites » ne disait plus vrai
  de cette puce.
- **Retenu** : « Deux précisions. » ; la première puce (boutons affichés, enregistrement refusé) est inchangée.
- **Réversible** : oui.

### C-29-4-37 — La moitié W25 de T29b-2 tue aussi M29-13, par son montage

- **Contexte** : la fiche (R5 de la P2) écrit que la moitié W25 ne tue pas M29-13, l'issue de la clôture (`Done`) étant
  la même sous la mutation.
- **Retenu** : le patron commun de T29b-2 **exige** que la clôture soit vue en attente sur son `FOR UPDATE` (étape de
  l'AC7) ; sous M29-13, elle n'attend pas, et le montage échoue — pour les deux moitiés. Dit au doc-comment de la moitié
  W25 : c'est l'attente, non l'issue, qui distingue.
- **Réversible** : sans objet (constat).

### C-29-4-38 — Le volet inverse « en dernier », jeton par jeton : une table de cas écrite à part des listes

- **Contexte** : revue de code P1 de la 29-4b, B-1 (MEDIUM) — le volet inverse de T29b-4 n'éprouvait que quatre des dix
  jetons de `GESTES_VERROUILLANTS_SQL` et `GESTES_VERROUILLANTS_CODE` ; retirer l'un des six autres laissait tout vert.
- **Retenu** : `the_last_acquisition_rule_sees_each_locking_token` — une table de cas (cinq littéraux, cinq appels)
  **écrite dans le test, indépendamment des listes**, un geste du seul jeton après la revérification, faute exacte
  « précède « <jeton> » » exigée ; décompte anti-muet (dix cas) ; puis contrôle de couverture dans les deux sens (chaque
  entrée des listes a son cas). Retirer un jeton d'une liste rougit **par son cas** (« aucun geste verrouillant ») ;
  ajouter un jeton sans cas rougit par la couverture. Prouvé : M-B1a (`INSERT INTO`) et M-B1b (`reverse_in_tx(`)
  tuées (`294b-mutations.log`).
- **Écartée** : boucler sur les listes elles-mêmes pour construire les cas — retirer un jeton retirerait son cas, et le
  test resterait vert.
- **Réversible** : oui.

### C-29-4-39 — T29b-1 exige que l'annulation soit vue en attente

- **Contexte** : revue de code P1 de la 29-4b, E-4 et A-9 — `annuler_pendant_une_cloture` ignorait le résultat
  d'`attendre_une_requete_en_cours`, qui rend `false` quand la tâche a fini sans attendre (et panique à dix secondes).
- **Retenu** : `false` fait rougir (la clôture est d'abord annulée, puis le test échoue en citant l'issue de
  l'annulation). Au code corrigé, l'annulation **doit** attendre : la clôture tient l'exclusif de la ligne que la
  revérification demande. Le doc-comment du patron ne dit plus « ou terminée ».
- **Écartée** : `attend || tache.is_finished()` (l'issue « finie sans attendre » est précisément celle où la garde de
  tête, non la revérification, aurait pu refuser).
- **Réversible** : oui.

