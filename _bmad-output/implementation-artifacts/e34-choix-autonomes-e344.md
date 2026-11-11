# Epic 34 — registre des choix autonomes, développement de la 34-4 (clé e344)

Choix faits au développement de la Story 34-4 (#828), en autonomie. Les choix de la spécification sont au registre du
groupe, `e34-choix-autonomes-e342.md` (C-e342-5 à C-e342-9, C-e342-20 à C-e342-23).

### C-e344-1 — Tests et banc dans `reconciliation_link_entry_e2e.rs`

- **Contexte** : la fiche demande des tests du dépôt (trois comptes, comptes partagés, contre-passation), un test HTTP et un
  banc ; le montage (serveur, écritures par la route, comptes bancaires liés, transactions par le dépôt) existe déjà dans ce
  binaire de test, et chaque binaire recopie le sien (C-v017-3-40).
- **Retenu** : ajouter les tests (`s34_4_…`) et le banc (`banc_cout_propositions_deja_comptabilisees`, `#[ignore]`) à ce
  fichier, en réutilisant `Ctx`. Les tests passent par la route (`GET /reconciliation/proposals`) et comparent aux deux routes
  du dialogue : ils voient la frontière HTTP, et la même assertion porte AC-1 et AC-2. Le test de l'égalité des textes SQL est
  un test unitaire de `reconciliation_link.rs` (le constructeur est privé).
- **Écartée** : un nouveau binaire qui recopierait le montage (≈ 500 lignes de plus, contraire à DRY).
- **Réversible** : oui.

### C-e344-2 — Un compte bancaire sans compte du grand livre compte comme « grand livre distinct »

- **Contexte** : AC-2 dit « dont le compte du grand livre diffère de celui de `t` » ; un compte bancaire peut avoir été délié
  après qu'une de ses transactions a été rapprochée.
- **Retenu** : `NOT (ba.journal_account_id <=> ?)` : `NULL` diffère du compte de `t`. L'écriture rapprochée porte un mouvement
  sur le grand livre de `t` et a été rapprochée ailleurs : c'est le second côté probable.
- **Écartée** : `ba.journal_account_id <> ?`, qui écarte en silence le cas `NULL`.
- **Réversible** : oui (un opérateur).

### C-e344-3 — Le nom du bouton entre guillemets dans les textes

- **Contexte** : la fiche écrit *Rattacher à une écriture existante* en italique Markdown ; un message du catalogue est du texte
  brut.
- **Retenu** : « Rattacher à une écriture existante » entre guillemets français (allemand „…“, anglais "…", italien «…»).
- **Réversible** : oui.

### C-e344-4 — Une ligne est « signalée » exactement quand elle porte un avertissement

- **Contexte** : AC-6 dit « au moins une ligne signalée » sans définir le terme ; AC-5 définit l'avertissement.
- **Retenu** : `isAlreadyBooked(p) = alreadyBookedWarning(p) !== null` (`already-booked.ts`) : une seule condition, `k > 0` ou
  `m > 0`. La note de la fiche pour la 34-11b (« la confirmation reste déclenchée par `k > 0` ») vaudra après sa réécriture,
  où `m ⊂ k`.
- **Écartée** : deux conditions séparées, qui pourraient diverger.
- **Réversible** : oui.

### C-e344-5 — Textes du dialogue de confirmation

- **Contexte** : la fiche fixe les boutons (*Continuer quand même*, *Annuler*) et le contenu (chaque transaction signalée,
  date suisse, montant, contrepartie), pas le titre ni la phrase d'introduction.
- **Retenu** : titre « Transaction peut-être déjà comptabilisée » ; introduction « Kesh signale ces transactions comme semblant
  déjà comptabilisées (voir l'avertissement de leur ligne). Continuer passerait une écriture pour un mouvement qui est
  peut-être déjà aux livres : il serait compté deux fois. » Fermer le dialogue (Échap, clic hors) vaut *Annuler*.
- **Réversible** : oui.

### C-e344-6 — L'avertissement dans une rangée propre, sous la ligne

- **Contexte** : AC-5 dit « sous la ligne ».
- **Retenu** : une seconde `<tr data-testid="already-booked-row">`, cellule vide sous la case, puis `colspan="6"` ; encadré ambre
  comme un avis. Les sélecteurs `reconciliation-row` ne la comptent pas.
- **Écartée** : l'avertissement dans la cellule de la candidate (il serait masqué par l'avis du lot, `notice`, qui la remplace).
- **Réversible** : oui.

### C-e344-7 — La désignation d'une transaction, partagée

- **Contexte** : la confirmation nomme une transaction comme le bilan d'un lot (`describeTx`, Story 15-5c).
- **Retenu** : `describeTransaction` dans `already-booked.ts`, appelé par les deux (DRY).
- **Réversible** : oui.

### C-e344-8 — Le banc : historique forgé par SQL, mesure d'avant sur les sources de base

- **Contexte** : AC-8 veut 1 000, 5 000 et 20 000 écritures et 500 transactions, mesurés avant et après le champ.
- **Retenu** : écritures insérées par SQL forgé (deux lignes chacune, montants `(n mod 997) + 1`, signe alterné ; aucune
  rapprochée) ; transactions par `bank_imports::create_with_transactions`. La mesure d'« avant » se joue sur le même banc, les
  deux fichiers sources du serveur remis temporairement à `f12ad73e` (`git restore --source`), puis restaurés
  (`kesh-gate-logs/e344/scripts/rouges-et-bancs-e344.sh`).
- **Écartée** : écritures par la route (des minutes par palier) ; un second worktree pour l'« avant » (20 à 50 Go de cible).
- **Réversible** : oui.

### C-e344-9 — Le remède de l'AC-8 : deux lectures par page, sur les mêmes textes SQL

- **Contexte** : le banc a mesuré, avec trois lectures par transaction (première écriture du code), ≈ 18,7 s pour 500
  transactions à 20 000 écritures (≈ 0,3 s avant le champ) : « mauvaise pente » au sens de l'AC-8 (plus de 3 s). Chaque
  lecture agrège toutes les lignes du compte du grand livre.
- **Retenu** : `count_already_booked_for_page` — une lecture des règlements rattachables du compte (leur seul `M`), une
  lecture des écritures dont `M` est l'un des montants de la page (`m.movement IN (…)`), avec deux colonnes « non
  rapprochée » et « rapprochée ailleurs » ; les tests de montant, la tolérance et le plafond de 50 passent en Rust, en
  décimal exact. Les conditions SQL ne sont pas recopiées : elles sont extraites en constantes partagées avec les requêtes du
  dialogue (`LINKABLE_SETTLEMENTS`, `ENTRY_MOVEMENTS`, `NOT_MATCHED`, `MATCHED_ELSEWHERE`,
  `ENTRY_NOT_REVERSED_NOR_SETTLEMENT`). Le test `s34_4_ac8_la_lecture_par_page_egale_les_fonctions_du_dialogue` compare,
  pour 50 transactions variées, les trois nombres aux longueurs des trois fonctions par transaction. Banc après remède :
  ≈ 0,3 s à 20 000 écritures. `find_matched_elsewhere` (le mode du constructeur) reste la référence du second côté ; la
  fonction par transaction est retirée.
- **Écartées** : garder trois lectures par transaction (mauvaise pente) ; une dette de performance (le remède rend les
  nombres exacts).
- **Réversible** : oui.

### C-e344-10 — Formatage par `cargo fmt`

- **Contexte** : la consigne « l'outil Edit seulement » vise les remplacements par script.
- **Retenu** : `cargo fmt --all` a reformaté les fichiers Rust touchés (le formateur du dépôt, exigé par la CI) ; toutes les
  modifications de fond sont faites par l'outil Edit. Écart déclaré au Change Log.
- **Réversible** : oui.
