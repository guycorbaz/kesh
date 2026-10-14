# Epic 29 — registre des choix autonomes, Story 29-1 (à verser par union dans `epic-29-choix-autonomes.md`)

*Fiche : `29-1-preparer-cloture.md`. Choix faits le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur
`16eeed21`. Contexte, retenu, écartées, réversibilité.*

---

## Story 29-1 — Préparer la clôture d'un exercice (#232)

### C-29-1-1 — Une fiche, deux stories (29-1a contrôles préalables, 29-1b transitoires) ; pas de 29-1c

- **Contexte** : l'ensemble touche plus de cinq modules (§ *Règle de splitting préventif*) ; le fichier d'epic
  prévoyait 29-1a / 29-1b / « éventuellement 29-1c affectation du résultat ».
- **Retenu** : précédents C-26-2-15 et C-28-4-1 — une fiche, deux clés, deux PR successives, clé mère `split`.
  29-1a (lecture seule, `refs #232`) puis 29-1b (contre-passation au début de l'exercice suivant, E2E de parcours,
  répétition à blanc au manuel d'administration, `closes #232`). Chacune compte cinq modules, la borne de la règle.
  Pas de 29-1c : l'affectation du résultat est écartée (C-29-1-2) ; la répétition à blanc va à la dernière
  sous-story, la 29-1b, comme le demande l'orchestrateur.
- **Écartées** : une story unique (trop large) ; trois stories avec une 29-1c « répétition à blanc » seule
  (documentation sans code, qui se relit avec la 29-1b dont elle décrit les gestes).
  *(Rectifié en validation P2, 2026-10-14, décision de l'orchestrateur : la répétition à blanc **quitte** la 29-1b et
  devient la Story 29-7 — C-29-1-23. Le découpage 29-1a / 29-1b tient.)*
- **Réversible** : oui.

### C-29-1-2 — Ni écritures de clôture, ni écriture d'affectation du résultat

- **Contexte** : #232 demande la « génération des écritures de clôture (virement du résultat au bilan) » ;
  l'orchestrateur propose d'examiner une écriture d'affectation du résultat. Le modèle « temps réel virtuel »
  (Story 14-1, décision de Guy du 2026-07-21) calcule résultat de l'exercice et résultat reporté ; le compte 2979
  (`CurrentYearResult`) est non imputable.
- **Retenu** : rien n'est généré. Une écriture de virement doublerait le calcul et fausserait le bilan. Pour une
  personne physique ou un indépendant, l'affectation est sans objet (le résultat s'accumule aux fonds propres, ce que
  la ligne « Résultat reporté (calculé) » montre). Pour une société, l'affectation votée par l'assemblée reste une
  écriture **manuelle**, déjà possible (débit du compte physique de rôle `RetainedEarnings`, crédit dividendes ou
  réserve) ; sa **présentation** au bilan (compte physique négatif à côté de la ligne calculée positive) est
  perfectible — **demande d'issue** à l'orchestrateur, pour que la fermeture de #232 ne la fasse pas disparaître.
  *(Validation P1, 2026-10-14 : l'issue existe — **#667**, « Affectation du résultat d'une société — écriture de
  décision d'assemblée et présentation au bilan », `enhancement`, sans label de priorité à cette date : à poser.)*
- **Écartées** : écriture d'affectation assistée (double un calcul, ou exige de revoir le modèle 14-1) ; retour à des
  écritures de clôture physiques (décision d'architecture contraire).
- **Réversible** : oui (une story future peut ajouter l'affectation assistée pour les sociétés).

### C-29-1-3 — Les contrôles signalent, ils ne bloquent pas ; `fiscal_years::close` n'est pas touchée

- **Contexte** : candidats de l'orchestrateur (contrôles en lecture seule, utilisables en cours d'année) ; leçon B5
  de la rétrospective 28 (toute simplification qui touche une transaction se relit sur les verrous et la naissance de
  la vue).
- **Retenu** : une lecture séparée, rappelée dans le dialogue de clôture, sans effet sur le `POST /close`. Le bouton
  *Clôturer* reste actif ; seul K1 (antérieur ouvert) bloque, et il bloquait déjà. Aucun verrou, aucune lecture ne
  s'ajoute à la clôture.
- **Écartées** : refuser la clôture tant qu'un contrôle est en `attention` (la clôture se défait par une réouverture
  motivée ; un refus côté serveur obligerait à lire les contrôles **sous** les verrous de la clôture — exactement le
  terrain de la 28-4) ; une case « j'ai vu les avertissements » envoyée au serveur (cérémonie sans garantie).
- **Réversible** : oui.

### C-29-1-4 — Sept contrôles, et ceux qui sont écartés

- **Retenu** : K1 antérieur ouvert, K2 transactions bancaires non rapprochées, K3 relevé de fin d'année (manquant,
  arrêté trop tôt, en écart), K4 factures en brouillon, K5 factures fournisseur importées à compléter, K6 écritures
  incohérentes héritées (déséquilibrées, moins de deux lignes, date hors de leur exercice, ou d'un autre exercice
  datées dans celui-ci), K7 exercice suivant absent.
- **Écartés**, un par un :
  - *comptes d'attente soldés* : Kesh n'en a pas (aucun rôle, drapeau ni compte ; inventaire au code) ; un rôle neuf
    serait une migration breaking (un binaire antérieur ne décode pas un rôle inconnu) — prémisse de #232 corrigée ;
  - *TVA réconciliée* : report assumé des P1 de la TVA (utilisateur non assujetti) ; demande d'issue ou de
    rattachement au jalon « Vague 2 » *(validation P1, 2026-10-14 : noté en commentaire sur **#398**, jalon « Vague
    2 », sans issue neuve)* ;
  - *équation du bilan* : K6 en couvre les deux causes (écriture déséquilibrée ; ligne datée dans un exercice et
    rattachée à un autre — le compte de résultat filtre `fiscal_year_id`, le bilan non) ; un contrôle de l'équation
    répéterait K6 sans dire quelle écriture corriger ;
  - *verrou de période contre fin d'exercice* : la clôture fige tout l'exercice, le verrou n'ajoute rien à signaler ;
  - *postes ouverts débiteurs / créanciers* : une créance ouverte au 31 décembre est normale, l'écran *Postes
    ouverts* les liste déjà à une date ;
  - *factures fournisseur saisies ouvertes* : déjà comptabilisées (dette), rien d'anormal.
- **Réversible** : oui (un contrôle s'ajoute par un changement de la fiche et du test d'ensemble clos T-a1).

### C-29-1-5 — Lecture sous `ReadSnapshot`, tout rôle, sans audit

- **Contexte** : sept requêtes qui doivent voir la base au même instant ; les rapports lisent sur le pool sans
  instantané ; `ReadSnapshot` (Story 28-3) est conçu pour être réutilisé.
- **Retenu** : une connexion, une vue, `READ ONLY` ; route dans `authenticated_routes` (Consultation lit, comme les
  rapports) ; pas d'audit (lecture, pas un export).
- **Écartées** : lectures sur le pool (incohérence possible entre contrôles) ; audit `fiscal_year.checks_viewed`
  (bruit dans une piste de contrôle).
- **Réversible** : oui.

### C-29-1-6 — Le module `kesh-db/src/repositories/closing_checks.rs` et la route sous `/fiscal-years/{id}`

- **Retenu** : la lecture vit à côté des dépôts qu'elle interroge ; la route est une ressource de l'exercice.
- **Écartées** : `kesh-report` (états comptables, pas contrôles de données) ; `/reports/closing-checks?fiscalYearId=`
  (ce n'est pas un rapport exportable).
- **Réversible** : oui.

### C-29-1-7 — K3 réutilise la requête du tableau de bord, bornée, sans copie

- **Contexte** : seule `list_by_company_with_balances` sait corriger l'écart des décalages date de valeur /
  comptabilisation ; elle ne prend pas de date.
- **Retenu** : extraction d'une fonction privée au SQL **unique**, avec une borne facultative sur `period_to` ; le
  tableau de bord passe `None`, résultat inchangé (tests existants verts sans modification).
- **Écartées** : copier la requête (règle DRY ; deux copies divergeraient au prochain correctif) ; comparer seulement
  le dernier relevé sans borne (un relevé de janvier N+1 masquerait l'absence de celui de décembre).
- **Réversible** : oui.

### C-29-1-8 — Les transitoires : un geste « contre-passer au début de l'exercice suivant », sans marque ni migration

- **Contexte** : #232 demande « saisie des actifs/passifs de régularisation avec contre-passation automatique à
  l'ouverture de N+1 ». Kesh n'a pas d'événement « ouverture » (l'exercice suivant est créé par l'utilisateur, souvent
  avant la clôture de N) ; la contre-passation existe, datée du jour.
- **Retenu** : l'écriture de régularisation se saisit comme toute écriture ; un bouton de sa fiche la contre-passe au
  **premier jour de l'exercice qui suit le sien**, par la contre-passation unique (liaison `reverses_entry_id`,
  lettrage, refus, rejeu). « Automatique » s'entend : Kesh construit, date et lie l'inverse ; l'utilisateur déclenche.
- **Écartées** : (a) marque « transitoire » à la saisie + génération à la clôture de N — écrit sous les verrous de la
  clôture (cycle neuf avec les écrivains de N+1, terrain de la 28-4 et de #651), migration, et génération en masse qui
  décide seule ; (b) marque à la saisie + création de l'inverse dans la même transaction — l'inverse naîtrait avant
  que l'origine soit relue, et l'exercice suivant doit exister au moment de la saisie ; (c) contre-passation à date
  libre (C-29-1-9).
- **Réversible** : oui (une marque pourra s'ajouter plus tard, la machinerie restera).

### C-29-1-9 — Une seule date neuve : le premier jour de l'exercice suivant

- **Contexte** : le manuel fonde le verrou de période sur « la contre-passation est datée du jour : elle corrige
  depuis le présent ».
- **Retenu** : `ReversalDating { Today, NextFiscalYearStart }`. La date neuve tombe dans un exercice **postérieur** à
  celui de l'origine et passe par **tous** les refus de `create_in_tx_inner` (période verrouillée comprise).
- **Écartées** : une date choisie par l'utilisateur (rouvrirait la question du verrou de période, contre-passation
  datée dans le passé de l'origine).
- **Réversible** : oui.

### C-29-1-10 — Une route neuve, pas un corps sur `/reverse`

- **Retenu** : `POST /api/v1/journal-entries/{id}/reverse-at-next-fiscal-year-start`, sans corps.
- **Écartées** : un corps facultatif `{ "dating": … }` sur `/reverse` — les clients actuels envoient `{}` ou rien,
  et un extracteur JSON rendu obligatoire les casserait ; un paramètre de requête (un geste d'écriture distinct mérite
  une route nommée, inscrite au registre et rejouée).
- **Réversible** : oui.

### C-29-1-11 — L'exercice suivant : lecture ordinaire après `reversal_blockers`, confirmée par lecture verrouillante

- **Contexte** : B5 et § 4.2 de la rétrospective 28 — une lecture ordinaire avancée fait naître la vue plus tôt.
- **Retenu** : la vue naît déjà à `reversal_blockers` (première lecture ordinaire de la contre-passation) ; la lecture
  de l'exercice suivant se place **après**, au point où `today` est calculé ; `find_open_covering_date` (verrouillante,
  état validé le plus récent) confirme. Angle mort écrit au doc-comment : un exercice **intercalé** créé après la vue
  n'est pas vu.
  *(Rectifié en validation P1, 2026-10-14, F4 / R14 : « première lecture ordinaire » n'est vrai que **sur ce chemin**,
  appelé avec `authority = None` ; avec une autorité fournisseur, l'étape (1-bis) est une lecture ordinaire
  antérieure. La phrase ne s'énonce plus en général — AC11 de la fiche. Et la **sûreté** repose sur la confirmation
  verrouillante de N+1, non sur le verrou de N — AC12, C-29-1-20.)*
- **Écartées** : une lecture verrouillante par intervalle pour trouver le suivant (ordre du parcours fixé par
  l'optimiseur, verrous de clé suivante — la doc de `fiscal_years::close` explique pourquoi la clôture y a renoncé).
- **Réversible** : oui.

### C-29-1-12 — Refus `NEXT_FISCAL_YEAR_MISSING` et `NEXT_FISCAL_YEAR_CLOSED`, en 409

- **Retenu** : deux codes distincts — l'un appelle à créer l'exercice, l'autre à le rouvrir ou à renoncer —, 409
  comme les autres refus d'état d'exercice (`EARLIER_FISCAL_YEAR_OPEN`).
- **Écartée** : réutiliser `FISCAL_YEAR_INVALID` (400, « aucun exercice ouvert ne couvre cette date ») — ne dit ni
  quelle date ni quoi faire.
- **Réversible** : oui (codes neufs, aucun client ne les attend encore).

### C-29-1-13 — E2E de parcours et répétition à blanc dans la 29-1b

- **Contexte** : Guy ne testera le bouclement qu'en début 2027 ; aucune recette en usage ne rattrapera ce que les
  revues manquent.
- **Retenu** : un E2E de parcours complet (contrôles, rapprochement, transitoire, contre-passation, bilans N et N+1,
  clôture, réouverture) ; au manuel d'administration, une marche de répétition sur une **instance séparée** à partir
  d'un `.keshbackup`, sans rien rapporter vers la production.
  *(Rectifié en validation P2, 2026-10-14 : l'E2E de parcours reste dans la 29-1b ; la répétition à blanc en est
  retirée — Story 29-7, C-29-1-23.)*
- **Réversible** : oui.

### C-29-1-14 — Angle mort assumé : une transitoire oubliée n'est pas repérée

- **Contexte** : sans marque (C-29-1-8), rien ne distingue une écriture de régularisation d'une autre.
- **Retenu** : écrit au manuel (AC17). Pas une dette au sens du `CLAUDE.md` (aucun défaut : une fonctionnalité non
  promise) ; si Guy la veut, CR à ouvrir.
- **Réversible** : oui.

### C-29-1-15 — Le défaut voisin « (si clôturé) » du manuel est corrigé dans la 29-1a

- **Contexte** : `user-manual.tex:728` affirme que les comptes du bilan sont reportés « depuis le bilan de clôture de
  l'exercice précédent (si clôturé) » ; au code, le report est calculé que l'exercice précédent soit clos ou non.
- **Retenu** : correction **mécanique** (une parenthèse retirée, une phrase juste), dans le commit du manuel de la
  29-1a (B3 : un défaut voisin mécanique s'intègre).
- **Réversible** : oui.

---

## Choix ajoutés à la remédiation de la validation P1 (2026-10-14, Opus 5.5)

### C-29-1-16 — K3 : population par l'activité, précédence écrite, comptes non vérifiables sans effet sur le statut

- **Contexte** : P1 R5 — la population d'origine (tout compte non archivé ni clos avant Y) faisait signaler chaque
  année un compte alimenté par CSV seulement, un compte ouvert après Y et un compte sans mouvement ; la précédence
  entre « sans relevé » et « non comparable » n'était pas écrite.
- **Retenu** : compte examiné = non archivé, non clos avant Y, **avec une activité dans Y** (une transaction bancaire
  ou un relevé à solde daté dans Y). Motif = le premier de `NOT_COMPARABLE` > `NO_BALANCE_STATEMENT` > `MISSING` >
  `ENDS_BEFORE_YEAR_END` > `GAP`. Les deux premiers vont dans une liste `notCheckable`, affichée, **sans effet sur le
  statut** ; les trois suivants font l'`attention`.
  *(Rectifié en validation P3, 2026-10-14, P3 R5 : la définition de l'activité ci-dessus est **supplantée** par
  C-29-1-24 — une ligne d'écriture sur le compte de grand livre lié compte aussi ; et la source du motif
  `NO_BALANCE_STATEMENT` est précisée à la fiche (quatrième `EXISTS`, sans borne de date, P3 R8).)*
- **Écartées** : un critère `created_at <= Y.end_date` (faux pour une reprise de comptabilité : le compte se crée en
  N+1 et reçoit les relevés de N) ; exclure purement les comptes non vérifiables (« signaler, jamais taire » : on
  dit pourquoi un compte n'est pas contrôlé) ; `NOT_COMPARABLE` en `attention` (alerte permanente pour un partage
  légitime d'un compte de grand livre).
- **Réversible** : oui.

### C-29-1-17 — L'écart du relevé : une seule définition côté serveur, arrondie au centime comme le tableau de bord

- **Contexte** : P1 R6 — les lignes sont en `DECIMAL(19,4)`, le relevé en `DECIMAL(18,2)` ; le tableau de bord arrondit
  les deux termes au centime (`statementGap`, TypeScript), K3 comparait sans arrondi.
- **Retenu** : `bank_accounts::statement_gap` (Rust), arrondi par `Money::round_to_centimes` ; le panneau affiche la
  valeur du serveur sans recalcul ; le test Rust reprend les cas de `homepage.test.ts`. Le calcul TypeScript du
  tableau de bord reste (servir son écart depuis le serveur toucherait un sixième module) : **issue demandée**.
  *(Rectifié en validation P2, 2026-10-14, P2 R8 : l'argument « sixième module » était faux au grain où la fiche
  compte, et l'issue n'existait pas. Le tableau de bord lit désormais l'écart du serveur — C-29-1-25 ; la seconde
  définition disparaît, la demande d'issue tombe.)*
- **Écartées** : recopier `statementGap` dans le panneau (troisième définition) ; comparer sans arrondi.
- **Réversible** : oui.

### C-29-1-18 — Lectures d'exercice partagées : `EARLIER_OPEN_SQL` et `find_next_after`

- **Contexte** : P1 R13 — K1 recopiait le prédicat de `FIND_EARLIER_OPEN_SQL` ; K7 et l'étape (4) de la 29-1b
  cherchent le même « exercice suivant ».
- **Retenu** : `fiscal_years.rs` porte le texte sans verrou (`EARLIER_OPEN_SQL`, `NEXT_AFTER_SQL`) et ses lectures
  (`find_earliest_earlier_open`, `find_next_after`), sur le modèle `OPEN_COVERING_DATE_SQL` ; `close` exécute le
  même texte suffixé de `FOR UPDATE`, identique octet pour octet (test T-a15). K7 et la contre-passation appellent
  la même fonction.
- **Écartées** : deux requêtes voisines dans `closing_checks.rs` et `journal_entries.rs`.
- **Réversible** : oui.

### C-29-1-19 — Le rappel du dialogue de clôture compte les contrôles `info`

- **Contexte** : P1 R16 — K7 en `info` ferait dire « 1 point à vérifier » tant que N+1 n'existe pas.
- **Retenu** : tous les `attention` comptent : un exercice suivant absent empêche de contre-passer les transitoires,
  des brouillons et des pièces à compléter sont des points réels avant une clôture. `notCheckable` ne compte pas.
- **Écartée** : ne compter que `blocking` / `warning` / `anomaly` (masquerait K4, K5, K7 au moment même où ils
  servent).
- **Réversible** : oui.

### C-29-1-20 — T-b5 : la vraie fonction en transaction, deux ordres, N premier exercice

- **Contexte** : P1 F1 / F2 / R14 — « A s'arrête après l'étape (1) » n'était pas montable ; avec un exercice
  antérieur à N, la clôture de N+1 forme le cycle connu au lieu d'attendre ; aucune mutation n'était tuée par T-b5.
- **Retenu** : `reverse_at_next_fiscal_year_start_in_tx` public ; ordre (i) A va au bout sans valider, B (`close`)
  attend en (b') — test de comportement, ne tue aucune mutation (N tenu deux fois), déclaré ; ordre (ii) B monté à la
  main (effets de `close`), A (vraie fonction) attend à l'étape (1) puis rend `NEXT_FISCAL_YEAR_CLOSED` — tue M-b3 et
  M-b11. N est le premier exercice. L'AC12 et le doc-comment disent que la **sûreté** repose sur la confirmation
  verrouillante de N+1 ; le verrou de N n'en est qu'une conséquence.
  *(Rectifié en validation P3, 2026-10-14, P3 R5 : l'ordre (ii) **ne tue plus M-b3** — voir C-29-1-27 ; M-b3 est tuée
  par T-b3 seul. La mutation M-b11 est réécrite à la fiche : c'est le `JOIN fiscal_years fy` qui sort de la requête
  verrouillante de l'étape (1), P3 F2.)*
- **Écartées** : reconstituer à la main les étapes de la contre-passation (précédent du test 13 d — ne tue aucune
  mutation du code) ; ajouter une variante en transaction de `close` (hors périmètre, touche la clôture).
- **Réversible** : oui.

### C-29-1-21 — Répétition à blanc : marche exécutable, et le `.keshbackup` traité comme un secret

- **Contexte** : P1 R8 / F7 — `container_name` et ports littéraux, image `:latest`, compose de production à base
  externe, SMTP de production qui enverrait de vrais courriels, fichier d'export qui porte condensés de mots de passe,
  clés API et jetons.
- **Retenu** : autre machine de préférence ; sinon dossier distinct, copie de `docker-compose.yml` (jamais
  `docker-compose.prod.yml` avec le `DATABASE_URL` de production), `container_name` et port édités, image épinglée à
  la version de `/health`, `-p kesh-repetition`, `.env` propre sans `KESH_SMTP_*` ; à la fin, `down -v`, suppression
  du dossier (avec sa sauvegarde pré-import) **et** du `.keshbackup` exporté ; jamais de dépôt dans un dossier
  synchronisé ni un dépôt de sources ; chiffrement s'il quitte la machine.
- **Écartées** : « un autre projet docker-compose » sans plus (non exécutable) ; garder l'export « pour la prochaine
  fois » (un secret qui traîne).
  *(Caduc en validation P2, 2026-10-14 : l'AC19 est retirée de la fiche — C-29-1-23. Ce choix reste comme trace et
  comme point de départ de la Story 29-7, qui devra l'éprouver en l'exécutant ; P2 R1 et R2 montrent qu'il ne suffit
  pas en l'état.)*
- **Réversible** : oui.

### C-29-1-22 — Fiche d'écriture : un motif à la place du bouton absent

- **Contexte** : P1 R9 — la règle de la page (« les boutons sont absents… le motif est affiché à leur place ») ;
  l'oubli de créer N+1 est le cas d'usage central.
- **Retenu** : motif traduit selon l'état de la liste des exercices (aucun pendant le chargement ; indisponibilité sur
  échec ; « créez d'abord l'exercice suivant » ; « l'exercice suivant est clôturé »), Admin/Comptable seulement.
  L'échec de la liste ne retire pas *Contre-passer* (du jour).
- **Écartées** : bouton désactivé (la page refuse les boutons grisés) ; bouton affiché sans liste (la confirmation
  doit nommer la date et l'exercice).
- **Réversible** : oui.

---

## Choix ajoutés à la remédiation de la validation P2 (2026-10-14, Opus 5.5)

*C-29-1-23 à 26 consignent des **décisions de l'orchestrateur**, transmises dans le prompt de remédiation ; C-29-1-27
à 29 sont des choix de la remédiation.*

### C-29-1-23 — La répétition à blanc quitte la fiche : Story 29-7 (décision de l'orchestrateur)

- **Contexte** : signal D5 levé en P2 (P1 0 HIGH / 12 MEDIUM → P2 1 HIGH / 12 MEDIUM, 13 des 16 findings R nés de la
  remédiation P1). L'AC19 a produit un défaut à chaque correction (P1 R8, F5, F7 ; P2 R1 HIGH, R2, F5), dont un qui
  pouvait faire remplacer la base de production par l'import (sur Synology, le compose de production s'enregistre
  sous le nom `docker-compose.yml`).
- **Retenu** : l'AC19, ses tâches et ses textes prescrits du manuel d'administration sont **retirés** — non réécrits ;
  l'AC19 garde son titre en renvoi. Story **29-7** (doc), clé `29-7-repetition-a-blanc: backlog`, à écrire **en
  l'exécutant réellement** sur la station avec une sauvegarde synthétique ; cahier des charges : P2 R1, R2 et F5.
- **Écartées** : une troisième réécriture de la marche dans la fiche (le motif du dépôt : chaque remédiation y
  produisait le défaut suivant, et seule l'exécution le révèle) ; une 29-1c doc (la marche se relit mieux exécutée
  qu'adossée à une fiche de code).
- **Réversible** : oui.

### C-29-1-24 — K3 lit aussi le grand livre (décision de l'orchestrateur)

- **Contexte** : P2 R3 = F2, convergents — un compte mouvementé au grand livre sans import bancaire dans Y échappait
  à K3, alors que son solde de fin d'année n'est confronté à aucun relevé.
- **Retenu** : compte examiné s'il a, dans Y, une transaction bancaire, un relevé à solde **ou** une ligne d'écriture
  sur son compte de grand livre lié ; trois `EXISTS` séparés (jamais `bank_transactions` dans le même `FROM` que les
  lignes) ; cas de T-a5, mutation M-a20, phrase du manuel écrite telle quelle.
- **Écartée** : écrire l'angle mort au manuel (« activité bancaire seulement ») — il tait le cas même que K3 doit
  signaler.
- **Réversible** : oui.

### C-29-1-25 — Le tableau de bord lit l'écart servi par le serveur (décision de l'orchestrateur)

- **Contexte** : P2 R8 — l'argument « sixième module » de C-29-1-17 était faux au grain de la crate, et l'issue
  demandée n'existait pas ; deux définitions de l'écart (Rust, TypeScript) restaient.
- **Retenu** : champ `statement_gap` dans `BankAccountWithBalance`, lu par `bank-accounts.api.ts` ; `statementGap` de
  `homepage.ts` lit ce champ, `toCentime` disparaît ; les cas d'arrondi passent en Rust (T-a14), test de route et de
  lecture neufs (T-a16), mutation M-a21. Une seule définition. Aucune crate neuve ; dispersion au grain fin déclarée.
- **Écartée** : garder le calcul TypeScript et ouvrir une issue (deux définitions qui divergeraient au prochain
  correctif).
- **Réversible** : oui.

### C-29-1-26 — T-a2 inversée : inventaire des sites non résolus (décision de l'orchestrateur)

- **Contexte** : P2 R4 = F1 — la garde « aucun `query*("` littéral » manquait le turbofish, l'appel coupé par
  rustfmt, `r#"…"#`, `concat!` et `format!` : verte sur la forme exacte qu'elle devait interdire.
- **Retenu** : tout site d'appel de requête du module a pour premier argument une constante de `ALL_SQL` (turbofish
  sauté en équilibrant les chevrons, source mis sur une ligne) — tout autre site est nommé comme non résolu ; recherche
  de `FOR UPDATE` / `LOCK IN SHARE MODE` / `FOR SHARE` dans le source hors tests **décommenté** ; `format!` et
  `concat!` en argument de requête sont des non résolus (non interdits ailleurs : un libellé peut s'y construire) ;
  liste close des fonctions externes ; cas de contrôle du détecteur ; mutation M-a19.
  *(Rectifié en validation P3, 2026-10-14, **décision de l'orchestrateur** — P3 R2, signal D5 au troisième tour sur
  cette garde : « une forme qui produit un défaut à chaque correction se retire ». T-a2 est **réduite à ses deux
  volets solides** : (a) tout appel de requête a pour premier argument une constante de `ALL_SQL` ; (b) aucun mot de
  verrou dans le source décommenté. Les volets (c) — liste close des fonctions externes, que les noms importés par
  `use` contournaient et qu'aucun cas de contrôle n'exerçait — et (d) — constantes externes sans verrou — **sortent
  du test**, sans rien qui les remplace : les fonctions de dépôt externes appelées par `closing_checks` sont nommées
  dans son doc-comment et vérifiées en revue de code, **angle mort assumé** écrit à l'AC2. Conséquence mécanique :
  `BALANCES_SQL`, `EARLIER_OPEN_SQL` et `NEXT_AFTER_SQL`, rendues `pub(crate)` pour le seul volet (d), redeviennent
  privées.)*
- **Écartée** : ajouter des formes à l'énumération (la classe resterait ouverte).
- **Réversible** : oui.

### C-29-1-27 — M-b3 : le statut de N+1 n'est jugé que par la confirmation ; T-b5 (ii) ne la revendique plus

- **Contexte** : P2 R5 = F3 — une implémentation qui juge `next.status` sur la lecture ordinaire rend
  `NEXT_FISCAL_YEAR_CLOSED` avec ou sans confirmation, dans T-b3 comme dans T-b5 (ii) ; l'état qui distinguerait la
  confirmation (« ordinaire : ouvert ; verrouillante : clos ») est inatteignable sous `close` actuel. L'orchestrateur
  laissait le choix entre un ordre (iii) où B ne tient que N+1 et le retrait de T-b5 (ii) des tueurs de M-b3.
- **Retenu** : l'AC11 **prescrit** que le statut n'est jugé que par la confirmation verrouillante ; M-b3 (confirmation
  retirée, aucun contrôle) est tuée par T-b3 seul ; T-b5 (ii) ne tue que M-b11 ; la variante « statut lu sur la
  lecture ordinaire » est M-b13, **déclarée équivalente**, exclue par la prescription et vérifiée à la lecture en
  revue de code ; l'AC12 et le doc-comment disent que la confirmation est une défense en profondeur hors d'atteinte de
  la course.
- **Écartée** : l'ordre (iii) — il monterait à la main un état qu'aucun code de Kesh ne produit (une clôture de N+1
  qui ne tient pas N), et un test qui tue une mutation sur un état fabriqué ne prouve rien du code réel.
- **Réversible** : oui (si `close` cesse un jour de verrouiller ses antérieurs, l'état devient atteignable et un ordre
  (iii) devient le bon test).
- *(Rectifié en validation P3, 2026-10-14, **décision de l'orchestrateur** — P3 R1 = F1, signal D5 au troisième tour
  sur cette famille (P1 F2 / R14 → P2 R5 = F3 → P3 R1 = F1) : « une forme qui produit un défaut à chaque correction se
  retire ». Le raisonnement ci-dessus ne regardait qu'un sens de divergence ; la réouverture de N+1, qui ne prend pas
  N, produit l'autre. **M-b13 est retirée** de la table des mutations, et rien ne la remplace : la fiche ne porte plus
  aucune affirmation d'équivalence ni d'inatteignabilité à son sujet, ni de « défense en profondeur hors d'atteinte ».
  Restent : M-b3, tuée par T-b3 ; la prescription de l'AC11 (« le statut de N+1 n'est jugé que par la confirmation
  verrouillante ») ; et, à l'AC12, la seule phrase que la réouverture de N+1 peut changer l'état lu entre la vue et la
  confirmation, et que c'est la confirmation qui décide. L'écartée « ordre (iii) » et la réversibilité ci-dessus
  sont caduques avec M-b13.)*

### C-29-1-28 — `api-external.md` et CHANGELOG de la 29-1b passent à l'AC17

- **Contexte** : l'AC19 retirée portait aussi ces deux livrables de la 29-1b.
- **Retenu** : l'AC17 les porte (route rejouée et deux codes d'erreur au § 10 de `api-external.md` ; entrée
  `### Ajouté` dans `## [0.14.0] — Non publié`) ; aucune renumérotation des AC (l'AC19 garde son titre en renvoi).
- **Écartées** : une AC20 neuve (renumérotation inutile) ; les répartir entre AC13 et AC14 (le CHANGELOG n'y a pas sa
  place).
- **Réversible** : oui.

### C-29-1-29 — Comptes de l'E2E de parcours créés par l'API à l'étape 0

- **Contexte** : P2 R7 = F4 — le préréglage `with-company-no-fy` ne pose ni compte 1300, ni compte bancaire, et son
  1100 est le compte débiteurs par défaut, refusé comme compte lié.
- **Retenu** : la spec crée `1300 Actifs transitoires` et `1020 Banque` (`Asset`, numéros et noms des plans livrés)
  puis un compte bancaire lié à 1020 ; la charge de l'étape 4 est `4000 Charges CI` ; réserve sur les comptes bancaires
  du préréglage retirée.
- **Écartées** : lier 1000 (accepté, mais compte d'impôt préalable par défaut du préréglage) ; enrichir le préréglage
  (touche `test_fixtures.rs` et tous les tests qui le comptent).
- **Réversible** : oui.

### C-29-1-30 — Fiche d'écriture : la liste des exercices lue à l'ouverture, `startEdit` garde sa propre lecture

- **Contexte** : P3 R7 — le bouton de l'AC15 a besoin de la liste des exercices dès l'ouverture de la page ; le mode
  édition (`startEdit`, `+page.svelte` l. 211-235) la lit déjà, mais au clic, par `Promise.allSettled`, avec sa
  dégradation propre (bornes de date absentes, jamais d'enregistrement bloqué).
- **Retenu** : une lecture neuve à l'ouverture, qui porte ses trois états (chargement, échec, succès) ; `startEdit`
  **inchangé**, relit au clic. Le mock existant de `journal-entry-page.test.ts` (`listFiscalYears: vi.fn()`, l. 49)
  reçoit un comportement par défaut (liste vide résolue) et les trois comportements de T-b8.
- **Écartées** : faire lire à `startEdit` la liste chargée à l'ouverture (elle a pu changer depuis, et son échec
  priverait le mode édition de ses bornes pour une raison qui n'est pas la sienne ; touche un chemin que la story n'a
  pas à toucher).
- **Réversible** : oui.
