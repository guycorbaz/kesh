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

### C-29-4-6 — Découper en 29-4a (import, lot, rejeu) et 29-4b (annulations, textes)

- **Contexte** : plus de cinq modules au total ; le risque n'est pas réparti également.
- **Retenu** : 29-4a sans verrou neuf d'écrivain (quatre modules) ; 29-4b porte le verrou neuf, son cycle et les textes
  (cinq modules, la borne). Une fiche, deux clés (précédent C-28-4-1) ; `refs #651` puis `closes #651` ; aucune release
  entre les deux (la v0.13.4 attend la 29-4b).
- **Écartées** : une seule story (dérogation, comme la 28-4a — non nécessaire ici, la moitié a est cohérente seule :
  l'inventaire de l'AC5 dit `Ouverte("29-4b")` pour les cinq sites restants) ; trois stories (le rejeu seul n'a pas
  d'objet sans la 29-4b ; il est rangé avec la fonction de revérification, même module).
- **Réversible** : oui.

### C-29-4-7 — Résidu : un cycle à trois où l'import serait victime

- **Contexte** : import (partagé depuis son `INSERT`, en attente de l'intervalle d'une suppression d'import), suppression
  d'import (revérification en attente derrière la clôture), clôture en attente : cycle à trois sur un même compte.
- **Retenu** : assumé, écrit au doc-comment de la revérification ; la clôture, plus légère, est la victime désignée
  (prémisse tenue par le dépôt, `rejeu_interblocage_e2e.rs:9-11`), et elle est rejouée. Classe LOW : aucune issue.
- **Écartée** : rejouer l'import (extraction de ~160 lignes sur deux chemins, CAMT et CSV) — hors périmètre minimal.
- **Réversible** : oui (une enveloppe à ajouter si le cas se produit).

### C-29-4-8 — Pas de revérification là où un invariant ou un verrou existant tient

- **Retenu** : W30, W1-W6 et `accept_one_*` (I1), W20 (`pay_in_tx` sous `FOR UPDATE`, et I2), W8, W27, W13-W15
  (`FOR UPDATE` de la ligne) : aucun appel ; le troisième inventaire du registre les classe `SansFenetre` avec leur
  raison.
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
