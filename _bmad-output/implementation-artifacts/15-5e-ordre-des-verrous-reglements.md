# Story 15.5e : Rejeu sur interblocage des flux d'écriture

## Status

split

⛔ **CORPS VIDÉ — cette fiche ne contient plus ni critères, ni tâches, ni inventaire.** Elle ne garde
que les pointeurs vers ses deux sous-stories et l'historique des passes qui ont conduit au découpage.
*(La définition du statut `split` l'impose ; précédents : 15-5, 15-1, 17-2.)* La version complète
d'avant découpage — validée P1 à P3 — se lit au commit `94f1365e`. Le nom du fichier, hérité du
premier objet de la story (« l'ordre des verrous des règlements », avant la réécriture C54), est
gardé pour ne pas casser les renvois.

## Les deux sous-stories

| | fiche | ce qu'elle porte | issues |
|---|---|---|---|
| **15-5e1** | `15-5e1-socle-rejeu.md` | **Socle (story-zéro)** : l'inventaire fermé des routes qui écrivent au journal (21 + 4 exemptées), gravé dans une seconde colonne du registre `audit_route_registry.rs` (statut transitoire `ARejouer` pour le rollout, volet (c) robuste) ; les deux enveloppes nommées (`kesh_db::retry::retry_on_deadlock`, `kesh_api::retry::retry_app_on_deadlock`, prédicat `is_app_deadlock`) ; le nom d'opération en champ du `warn!` de `retry_with` ; les **trois routes des issues** rejouées — validation d'une facture, règlement client, annulation d'un règlement client — avec leurs tests « route victime » sur une vraie 1213 et le test du prédicat ; ce que la 15-5d attend de l'ordre des verrous (doc-comment canonique, avance des réglages de la saisie fournisseur, « 5 bis », module `retry.rs`) ; la ligne du CHANGELOG | `closes #463`, `closes #491` ; `refs #536`, `refs #429` |
| **15-5e2** | `15-5e2-rejeu-des-autres-flux.md` | **Rollout** : les 18 autres routes `Rejouee` (14 à rejouer, 3 sites existants migrés vers les enveloppes, `post_accept` inchangée), revue fichier par fichier ; le registre sans `ARejouer` ; les commentaires d'ordre restants et l'inventaire au symptôme (étendu aux tests) ; le Pattern 5 ; `docs/api-external.md` § 10 ; le CHANGELOG étendu ; les manuels (#484) | `closes #536`, `closes #484` ; `refs #463`, `refs #491` |

⚠️ **L'ordre** : 15-5a → 15-5b → **15-5e1** → (15-5e2, 15-5d), ces deux dernières dans un ordre
libre (choix C65). La 15-5d dépend de la **15-5e1 seule** : rejeu de la validation, avance des
réglages de la saisie fournisseur, doc-comment canonique (numérotation (2 bis')) et « 5 bis » y sont
posés. Le rejeu de la saisie fournisseur et de la complétion d'import vient avec la 15-5e2 : si la
15-5d merge avant elle, ces deux routes portent ses verrous sans rejeu jusqu'à ce merge — une victime
y rend 500 comme aujourd'hui.

## Pourquoi le découpage

La validation P3 (Sonnet ×2) a relevé (finding **F3-2 MEDIUM**) que la dérogation de découpage écrite
dans la fiche (choix C56) ne reposait pas sur l'exception que la § *Règle de splitting préventif*
prévoit — cycles de dépendance Cargo, merges intermédiaires impossibles à tester en isolation : ses
deux raisons étaient « le patron existe » (contredit par l'AC2 : deux enveloppes neuves, un
changement de signature, la journalisation) et « des routes resteraient non rejouées entre deux
merges » (c'est l'état actuel de dix-sept routes, pas une impossibilité de tester). La story mêlait en
outre du code de patron, un rollout sur neuf modules de routes et une documentation à risque.
L'orchestrateur a découpé selon le patron de la règle — **story-zéro qui pose le patron sur les
routes pilotes, puis rollout mécanique** (choix **C61**). Les remédiations de la P3 ont été
appliquées **dans les deux fiches filles**, pas ici.

## Décisions prises pour la story et où elles vivent

| choix | objet | fiche |
|---|---|---|
| C43 | côté achat : réglages avancés avant les comptes de charge (le reste retiré par C55) | 15-5e1 |
| C52, C53 | création de la 15-5e par découpage de la 15-5d, ligne de partage (historique) | — |
| C54 | la défense contre l'interblocage est le rejeu, pas un ordre parfait des verrous | 15-5e1, 15-5e2 |
| C55 | ce qui reste de l'ordre : l'avance des réglages de la saisie fournisseur et des commentaires vrais | 15-5e1 (avance, doc-comment canonique, « 5 bis »), 15-5e2 (autres commentaires) |
| C56 | deux enveloppes partagées, registre des routes (dérogation de découpage révisée par C61) | 15-5e1 (pose), 15-5e2 (emploie) |
| C57 | #536 fermée par le rejeu | 15-5e1 (validation), 15-5e2 (avoir, `closes`) |
| C58 | le registre est une seconde colonne du registre d'audit | 15-5e1, 15-5e2 |
| C59 | rejeux journalisés en `warn` (révisé par C62) | 15-5e1 |
| C60 | #484 fermée par la 15-5e | 15-5e2 |
| C61 | découpage 15-5e1 / 15-5e2 | les deux |
| C62 | noms des enveloppes ; nom d'opération en champ de l'événement | 15-5e1 |
| C63 | statut transitoire `ARejouer` ; volet (c) robuste ; `retry_with` restreint à `post_accept` | 15-5e1, 15-5e2 |
| C64 | la ligne du CHANGELOG voyage avec les issues ; PDF de la brochure restauré | 15-5e1, 15-5e2 |
| C65 | la 15-5d dépend de la 15-5e1 seule | 15-5e1, 15-5d |

## Change Log

- 2026-10-08 — **Créée par le découpage de la 15-5d** (choix **C52**, ligne de partage **C53**), après
  sa validation P3, sous le titre « L'ordre des verrous des règlements — l'arrondi d'abord, les
  réglages avant les comptes » : réalignement de trois flux existants (règlement client par compte
  interne, solde du reste, saisie fournisseur) sur l'ordre canonique des verrous, ordres de refus qui
  en découlent, trois tests de concurrence à sonde `NOWAIT`, doc-comment canonique et commentaire
  « 5 bis ». Contenu validé dans la 15-5d : P2 (Opus ×2 ; F2-1 HIGH = R2-1, F2-8 LOW ; choix **C43**),
  P3 (Sonnet ×2 ; R3-1 = F3-1, F3-2 MEDIUM, F3-3, R3-2 LOW ; choix **C48**). 7 AC, 6 tâches.
- 2026-10-08 — **Validation P1** (Sonnet ×2, lentilles R et F, contexte frais, lecture seule ; prompt
  versionné `15-5e-validate-prompt-p1.md` ; rapports `target/gate-logs/15-5e-p1-R.md` et `-F.md`) :
  **R : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 3 LOW ; F : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 3 LOW**.

  | finding | sévérité | constat | sort après C54 |
  |---|---|---|---|
  | F1-1 | HIGH | l'insertion des lignes reprend `accounts` par `fk_jel_account` après l'exercice : aucun ordre « comptes avant exercice » ne tient ; le cycle (d) n'est pas fermé | **à l'origine de C54** : la story devient le rejeu ; le doc-comment canonique nomme la reprise (AC5) |
  | F1-2 | MEDIUM | le Pattern 5 fixe l'ordre global inverse (comptes avant réglages) et n'est pas touché | AC5 : Pattern 5 réécrit (convention de fréquence, deux lignes de table corrigées) |
  | F1-3 | MEDIUM | #491 non citée ; doc de `write_off_invoice_handler` à réviser ; avoir dit « réglages lus sans verrou » | `closes #491` (AC3, AC4) ; doc du handler (AC5) ; cellule de l'avoir corrigée (Dev Notes) |
  | F1-4 | MEDIUM | la concurrence 1 ne prouve rien pour sa seconde tâche | tests de concurrence retirés (C55) ; AC4 interdit les tests de « non-interblocage » |
  | R1-1 | MEDIUM | l'avoir verrouille ses réglages ; #536 porte déjà l'avoir | table des Dev Notes corrigée ; #536 fermée par le rejeu, l'avoir compris (C57) |
  | R1-2 | MEDIUM | inventaire des sites qui verrouillent `accounts` ouvert (deux noms de fonction) | sans objet : l'absence de cycle n'est plus affirmée (C55) ; l'inventaire fermé porte désormais sur les **routes** qui écrivent au journal, parti du symptôme (AC1) |
  | R1-3 | MEDIUM | comptes de charge verrouillés dans l'ordre des lignes ; un compte de charge peut être le compte d'arrondi | pas de tri, raison écrite (AC5, C55) ; le rejeu couvre le cycle restant |
  | F1-5 | LOW | « `validate_invoice` prend de même le projet » : faux, lecture sans verrou | AC5 : le doc-comment canonique dit « lu sans verrou » |
  | F1-6 | LOW | la sûreté des deux saisies repose sur le verrou des réglages, non écrit | AC5 : écrit (raison de l'avance des réglages) |
  | F1-7 | LOW | place des réglages vs les deux passes de la 15-5a ; étape réglages absente du doc-comment ; manuels admin non cités ; « aucune ne finisse en 500 » vaut pour trois flux sur quatre | AC5 : entre la passe de forme et celle des comptes ; étape réglages écrite ; plus de changement de refus, donc plus de manuel à toucher ; le rejeu porte sur toutes les routes |
  | R1-4 | LOW | prémisse du prompt (« 15-5a mergée ») invérifiable | sans effet sur la fiche ; la dépendance à la 15-5a reste écrite |
  | R1-5 | LOW | le helper d'attente ne distingue pas deux tâches aux mêmes motifs | sans objet (concurrence 1 retirée) ; les tests de l'AC4 n'attendent qu'une tâche |
  | R1-6 | LOW | partage de code virement / compte interne non spécifié | sans objet (ancien AC2 retiré) |

- 2026-10-08 — **Réécriture selon C54** (choix de l'orchestrateur ; choix d'application **C55**,
  **C56**, **C57** consignés). Titre neuf : « Rejeu sur interblocage des flux d'écriture » ; nom de
  fichier gardé. **La story change d'objet** : elle rejoue sur interblocage (1213) les 17 routes qui
  écrivent au journal sans rejeu — inventaire fermé parti des primitives d'écriture (AC1), deux
  enveloppes partagées et migration de trois sites existants (AC2, AC3), test du prédicat sur une
  vraie 1213, trois tests où la route est la victime (#491, #463, #536) et registre des routes
  (AC4). **Issues** : `closes #463 #491 #536` (au lieu de `refs #429` seul, qui reste). **De l'ordre
  des verrous ne restent** que l'avance des réglages de la saisie fournisseur (nécessaire à la 15-5d)
  et des commentaires qui disent la règle vraie — doc-comment canonique avec `fk_jel_account`,
  « 5 bis », `write_off_invoice_handler`, module `retry.rs` (dont l'affirmation fausse « ne détecte
  pas les deadlocks cross-table »), Pattern 5 (AC5). **Retirés** : « l'arrondi d'abord » au règlement
  client et au solde du reste, leurs priorités de refus, la ligne « Modifié » du CHANGELOG, les trois
  tests à sonde `NOWAIT`, les affirmations de cycles fermés. CHANGELOG : ligne « Corrigé » (#463,
  #491, #536) ; `docs/api-external.md` § 10. **Signal de découpage** (plus de cinq modules) déclaré
  au Project Lead, dérogation écrite (Dev Notes, C56). **Propagation post-patch** : `aucun cycle`,
  `ne s'interbloquent jamais`, `arrondi d'abord`, `NOWAIT`, `concurrence 1`, `sonde`, `Modifié`,
  `#429`, `cycle (`, `ferme` grepés sur le corps de la fiche (hors Change Log) : les occurrences
  restantes décrivent ce qui est retiré ou citent l'issue de la 15-5d. Faits revérifiés au code sur
  `b80ab8c0` : les cinq sites `retry_with`, les 21 routes et leurs lignes, les `map_err` des dépôts,
  la clé `fk_jel_account` (`20260412000001_journal_entries.sql:45`), les six occurrences du grep de
  l'AC5. Décompte : **7 AC, 6 tâches T0–T5** (recompté). **Validation** : une passe complète
  (**Opus**, rotation D6 après la P1 Sonnet) — la fiche est neuve dans son objet.
- 2026-10-08 — **Validation P2** (Opus ×2, lentilles R et F, contexte frais, lecture seule ; prompt
  versionné `15-5e-validate-prompt-p2.md` ; rapports `target/gate-logs/15-5e-p2-R.md` et `-F.md`) :
  **R : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 6 LOW ; F : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 6 LOW** — tous
  appliqués, arbitrages de l'orchestrateur consignés en **C58**, **C59**, **C60**.

  | finding | sévérité | constat | traitement |
  |---|---|---|---|
  | R2-1 | MEDIUM | la 15-5d étend une étape « 1 bis. `accounts` » que le doc-comment réécrit fait disparaître | AC5 : numérotation fixée sur les étapes du code ((1), (1 bis) projet, (2), (2 bis') arrondi, (2 quater), (3), (4)-(5), (7)) ; la 15-5d renvoie à **(2 bis')** (ses AC1 et AC9, Change Log) |
  | R2-2 | MEDIUM | Pattern 5 `:320-324` : « all current write endpoints follow the documented order […] deny list » survit au retrait de la deny list | AC5 : « Known Risk » réécrit ; `follow the documented order` et `[Dd]eny list` ajoutés à l'inventaire au symptôme |
  | R2-3 = F2-2 | MEDIUM | l'inventaire de l'AC1 ne voit pas le SQL à nom de table dynamique (`TRUNCATE`, `format!`) ; `/_test/seed`, `/_test/reset` manquent | AC1 et Dev Notes : troisième commande (relevé : 99 lignes, quatre sites qui écrivent) ; les deux routes de test **exemptées** (mode test) ; 4 exemptées au lieu de 2 |
  | F2-1 | MEDIUM | des commentaires fondent l'absence de cycle sur « l'ordre global companies → projects → fiscal_years », qu'`accept_one_rule` inverse ; la liste de formulations ne les voit pas | AC5 : grep à formulations remplacé par un **inventaire au symptôme** (148 lignes, 32 fichiers sur `c734645b`), trié occurrence par occurrence ; `journal_entries.rs:236-240`, `:253-260`, `:547-548` réécrits ; l'inversion d'`accept_one_rule` **et de `post_manual`** (trouvée en appliquant : exercice `:3124` puis sentinelle `:3136`, commentaire `:3128-3134` qui dit l'inverse) écrite ; `projects.rs`, `supplier_invoices.rs`, `reconciliation_rules.rs`, `journal_entry_number_sequences.rs` jugés vrais pour leur paire ; sentinelle et `fk_journal_entries_company`, `fk_jel_project` au Pattern 5 et à la table « pour mémoire » |
  | F2-3 | MEDIUM | le registre neuf dupliquerait l'extracteur d'`audit_route_registry.rs` | **C58** : seconde colonne dans le registre existant, deux tests ajoutés au même fichier ; plus de fichier `rejeu_route_registry.rs` |
  | F2-4 | MEDIUM | les manuels affirment des transactions `SERIALIZABLE` inexistantes (#484) ; le relevé de l'AC6 ne pouvait pas le voir | **C60** : `closes #484` ; AC6 réécrit les deux passages, PDF régénérés et contrôlés aplatis ; relevé élargi (15 lignes, triées) |
  | R2-4 | LOW | tri des sorties incomplet | Dev Notes : tri complet (homonymes, aides de démontage dont `delete_all_by_company`, faux amis du motif) |
  | R2-5 | LOW | `retry.rs:11-13` (« la plus jeune »), `:33-36` (50 s) hors de l'AC5 ; `:63-70` → `:67-70` | AC5 : trois prémisses réécrites ; ligne corrigée |
  | R2-6 | LOW | le verrou partagé de (2 quater) manque à « l'ordre réel » | AC5 : étape (2 quater) écrite |
  | R2-7 | LOW | « 5 bis » omet le compte de la nature | AC5 : nature → arrondi → TVA due → exercice |
  | R2-8 | LOW | `accept_once` `:879`, doc de `write_off_invoice_handler` `:1323-1326` | corrigés |
  | R2-9 | LOW | test 2 : mode de règlement non fixé, montage non partagé | AC4 : compte interne ; factures créées par les routes, aide locale, pas de troisième copie de `seed_validated_invoice` |
  | F2-5 | LOW | angle mort : une route sans écriture au journal peut être la victime | AC4 : écrit au doc-comment du registre, avec l'exemple `accept_batch` / `companies` |
  | F2-6 | LOW | « toutes les opérations » contredit les exemptions | AC6 : CHANGELOG et `api-external.md` nomment l'exception |
  | F2-7 | LOW | `retry.rs` doit dire ce qu'InnoDB ne détecte pas | AC5 : `GET_LOCK`, invariant « verrou nommé avant tout verrou de ligne » (vérifié en T0) ; `innodb_deadlock_detect` au manuel admin (AC6) |
  | F2-8 | LOW | rejeux invisibles de l'exploitant | **C59** : niveau `warn`, nom d'opération dans un span (AC2, T1) |
  | F2-9 | LOW | montages des tests 3 et 4 ; sentinelle omise à la ligne `POST /supplier-invoices` | AC4 : facture du jour, sans écart d'arrondi, exercice ouvert couvrant aujourd'hui ; AC5 : sentinelle gardée |
  | F2-10 | LOW | volet (c) du registre purement textuel | AC4 : écrit au doc-comment du registre (ce qu'il n'établit pas) |

  **Propagation post-patch** : `rejeu_route_registry`, `2 exemptées`, `1 bis`, `:878`, `:1322`,
  `:63-70`, `arrondi, puis TVA due`, `Aucun changement attendu`, `toutes les opérations qui
  écrivent`, `six occurrences`, `cinq tests`, `deny list`, `plus jeune`, `SERIALIZABLE` grepés sur le
  corps de la fiche : les occurrences restantes citent le texte à réécrire ou l'historique. Fiche
  15-5d : `1 bis` grepé, seule la mention historique de son Change Log reste. `SERIALIZABLE` grepé
  sur `crates/*/src`, `docs/`, `website/`, `README.md`, `CHANGELOG.md` : seulement les deux passages
  des manuels. **Recompte** sur `c734645b` (commandes exécutées) : inventaire au symptôme 148 lignes
  / 32 fichiers ; relevé dynamique 99 lignes ; relevé des manuels 15 lignes (3 lignes des deux passages, `905`,
  `885`, `886` + 1 vraie, `889` + 1 hors sujet, `1395` + 10 « isolation » au sens multi-société) ; routes : 21
  rejouées (4 + 17), 4 exemptées. Décompte de la fiche inchangé : **7 AC, 6 tâches T0–T5**.
  **Signal de découpage** (règle amendée D5) : la P2 trouve 7 MEDIUM contre 1 HIGH et 6 MEDIUM en
  P1 — sévérité maximale en baisse (HIGH → MEDIUM) ; les défauts sont **distincts** de ceux de la P1
  et viennent de l'inventaire et de la documentation, pas du recyclage d'un correctif (seul R2-1
  naît de la réécriture C54, qui a renuméroté le doc-comment) : pas de découpage, signal déclaré.
  **Prochaine passe** : P3 complète (Sonnet, rotation D6), la remédiation ayant touché plusieurs
  points de la fiche et ajouté #484.
- 2026-10-08 — **Validation P3** (Sonnet ×2, lentilles R et F, contexte frais, lecture seule ; prompt
  versionné `15-5e-validate-prompt-p3.md` ; rapports `target/gate-logs/15-5e-p3-R.md` et `-F.md`) :
  **R : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 7 LOW ; F : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 7 LOW** — tous
  traités, **puis DÉCOUPAGE** (choix **C61**, sur F3-2) en **15-5e1** (socle) et **15-5e2** (rollout) ;
  les remédiations sont appliquées dans les deux fiches filles. Choix consignés : **C61** à **C65**.

  | finding | sévérité | constat | traitement | fiche |
  |---|---|---|---|---|
  | F3-1 | MEDIUM | le volet (c) du registre est textuel : le doc-comment de la route suivante (qui nomme l'enveloppe, AC3) tombe dans la fenêtre du handler précédent — faux vert (`validate_invoice_handler` → `unvalidate`, `pay_supplier_invoice` → `cancel_supplier_invoice`) ; le nom de l'enveloppe `AppError` « au choix du dev » | **C62**, **C63** : enveloppes nommées ; commentaires retirés avant le match ; fenêtre coupée au premier attribut / doc-comment / item suivant ; recherche de `nom(` / `nom::<` ; extraction testée sur source synthétique ; mutations sur plusieurs familles (`validate_invoice_handler` suivi de la dévalidation, `retry_with` direct, nom en commentaire ; puis `pay_supplier_invoice` suivi de `cancel_supplier_invoice`, `post_manual`, `create_journal_entry`, `retry_with` recopié) ; limite (fonction auxiliaire) au doc-comment du registre | 15-5e1 (volet, mutations du socle), 15-5e2 (mutations du rollout) |
  | F3-2 | MEDIUM | la dérogation de découpage ne repose pas sur l'exception de la règle (cycles Cargo, merges non testables) | **C61** : découpage story-zéro + rollout ; cette fiche devient l'index (`split`) | toutes |
  | F3-3 | LOW | le nom d'opération porté par un span `info` disparaît sous `RUST_LOG=warn` | **C62** (révise C59) : champ `operation` du `warn!`, premier paramètre de `retry_with` | 15-5e1 |
  | F3-4 | LOW | l'inventaire au symptôme ignore `crates/*/tests` ; `fiscal_years.rs:497-502` non nommé ; `reconciliation_e2e.rs:4414` à trancher | périmètre étendu (181 lignes / 42 fichiers sur `94f1365e`) ; tri assumé des phrases de tests (`opening_complement_repository.rs:727`, `:781`, `fiscal_years_repository.rs:1071`) ; `fiscal_years.rs:497-502` réécrit ; `:4414` gardée (historique) | 15-5e2 |
  | F3-5 | LOW | le relevé des manuels ne voit pas `user-manual.tex:1532` (lot « atomique ») | motif étendu (`racing|race condition|atomi|verrous? de (ligne|base)|FOR UPDATE`) : 18 lignes ; `:1532` est réécrit par la 15-5c (AC6, #481), non touché ici | 15-5e2 |
  | F3-6 | LOW | registre : en-tête 111/114 contre assertions 112/115 ; partition sans total ; C56 dit encore « deux routes exemptées » | partition avec total (115) et part `SansEcritureAuJournal` (90) ; en-tête corrigé en T3 ; C56 marqué révisé par une entrée neuve (C61) | 15-5e1 ; registre |
  | F3-7 | LOW | (2 quater) écrit sans sa condition | « seulement si une ligne n'a pas encore de compte de produit » (`:2141-2148`) | 15-5e1 |
  | F3-8 | LOW | fermeture `Fn` : clonage des entrées non spécifié | règle posée (patron `finalize`, pas d'`Option::take`), appliquée route par route | 15-5e1 (règle), 15-5e2 |
  | F3-9 | LOW | CHANGELOG et manuel promettent plus que le mécanisme ; PDF de la brochure | réserve « seul un interblocage répété trois fois de suite… » au CHANGELOG et au manuel ; brochure restaurée après `make fr` (**C64**) | 15-5e1, 15-5e2 |
  | R3-1 | LOW | plage du doc-comment de `write_off_invoice_handler` décalée d'un cran (correction P2 inexacte) | `:1324-1327` ; ordre `:1324-1326`, `version` `:1326-1327` | 15-5e2 |
  | R3-2 | LOW | le changement de signature touche six appels de tests de `retry.rs` | écrits (`:189`, `:205`, `:225`, `:241`, `:318`, `:348`) ; plus les cinq sites directs de `retry_with` (C62) | 15-5e1 |
  | R3-3 | LOW | nom et visibilité de l'enveloppe `AppError` non fixés | `pub mod retry;`, `retry_app_on_deadlock`, `is_app_deadlock` exposé | 15-5e1 |
  | R3-4 | LOW | les contrôles de `complete_import` et la lecture de `cancel_reconciliation_once` dépendent de la transaction | règle : ce qui lit la transaction reste dans la tentative, dans le même ordre | 15-5e1 (règle), 15-5e2 (cas) |
  | R3-5 | LOW | compte rendu P2 : « 7 MEDIUM » compte deux fois R2-3 = F2-2 | rectifié ici : la P2 a trouvé **6 MEDIUM distincts (7 attributions)** et 12 LOW ; le signal de découpage de la P2 comparait donc 6 à 6 (1 HIGH et 6 MEDIUM en P1) | index |
  | R3-6 | LOW | résidus : C56 ; en-tête du registre ; `/password-reset-token` ; références route / dépôt ambiguës ; puce « CI lint » `:329` | C56 révisé par C61 ; en-tête corrigé ; `/password-reset-token` nommée `SansEcritureAuJournal` ; numéros de ligne préfixés route / dépôt ; puce « CI lint » retirée (le registre en tient le rôle) | 15-5e1, 15-5e2 |
  | R3-7 | LOW | « la ligne des réglages sérialise » probablement faux (`INSERT IGNORE` en doublon pose un S) — non exécuté | aucune promesse de sérialisation : « ordonne la plupart des flux ; un interblocage reste possible et il est rejoué » ; l'argument « pas de tri des comptes de charge » repose sur le rejeu | 15-5e1 |

  **Trend des passes** (sévérités distinctes, après fusion des doublons) : P1 **1 HIGH, 6 MEDIUM**,
  6 LOW → réécriture C54 → P2 **6 MEDIUM**, 12 LOW → P3 **2 MEDIUM**, 14 LOW attribués (7 + 7 ; recoupements partiels : R3-3 avec F3-1 sur le nom de l'enveloppe, R3-6 (a)-(b) avec F3-6). Modèles : Sonnet (P1),
  Opus (P2), Sonnet (P3), deux lentilles par passe. **Signal de découpage** (règle amendée D5) : la
  sévérité baisse et les défauts de la P3 sont distincts, mais F3-2 établit que la dérogation n'avait
  pas de fondement dans la règle — découpage (C61), décidé par l'orchestrateur. **Propagation
  post-patch** : `info_span`, `nom au choix`, `Deux routes exemptées`, `sérialise`, `111`, `114`,
  `1323`, `CI lint`, `crates/\*/src docs` grepés sur les deux fiches filles (hors Change Log) : les
  occurrences restantes citent le texte à corriger. Fiche **15-5d** : renvois à la 15-5e ventilés
  vers la 15-5e1 (sa dépendance) ou la 15-5e2 ; fiche mère **15-5** : table et ordre mis à jour.
  **Recompte** (commandes exécutées sur `94f1365e`) : inventaire au symptôme 148 + 33 = 181 lignes,
  32 + 10 = 42 fichiers ; relevé des manuels 18 lignes ; registre 112 + 3 = 115 routes (7 / 14 / 4 /
  90 à la 15-5e1, 21 / 4 / 90 à la 15-5e2). Décomptes des filles : **15-5e1 : 7 AC, 6 tâches** ;
  **15-5e2 : 6 AC, 6 tâches**. **Prochaine passe** : une passe complète (**Opus**, rotation D6) sur
  chacune des deux fiches filles, le découpage ayant redistribué tout le corps.
